//! A disk-backed extendible hash database in a single file.
//!
//! The directory is read into memory on open and written through on change.
//! Buckets are never cached: every `get` reads its bucket page from disk, and
//! every mutation writes the pages it changes, then the header, then (by
//! default) flushes the file to stable storage.
//!
//! A bucket is a chain of pages: one bucket page followed by zero or more
//! overflow pages. A bucket whose records no longer fit in one page is split,
//! as in the in-memory table. Overflow pages are used only when splitting
//! cannot help: the bucket is already at the maximum depth, or every record
//! in it has the same hash.
//!
//! See [`crate::format`] for the byte layout.

use std::fs::{self, File, OpenOptions};
use std::io;
use std::mem;
use std::path::Path;

use crate::error::{Error, Result};
use crate::format::{self, HEADER_LEN, Header, Page, PageKind, Record};
use crate::hash::{bit, key_hash, prefix};
use crate::pager::Pager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreateOptions {
    /// Bytes per page: a power of two from 512 to 32768.
    pub page_size: u32,
    /// The deepest the directory may grow, at most 32. Buckets at this depth
    /// grow overflow pages instead of splitting.
    pub max_depth: u8,
}

impl Default for CreateOptions {
    fn default() -> Self {
        Self {
            page_size: format::DEFAULT_PAGE_SIZE,
            max_depth: format::DEFAULT_MAX_DEPTH,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stats {
    pub page_size: u32,
    pub pages: u64,
    pub global_depth: u8,
    pub max_depth: u8,
    pub directory_size: usize,
    pub directory_pages: u64,
    pub buckets: u64,
    pub overflow_pages: u64,
    pub free_pages: u64,
    pub entries: u64,
    /// Size of the file on disk, in bytes.
    pub file_size: u64,
    /// Bytes of bucket and overflow pages occupied by records: their slots,
    /// keys, and values.
    pub record_bytes: u64,
}

impl Stats {
    /// How full the bucket and overflow pages are, from 0 to 100: record
    /// bytes as a share of the space those pages offer for records.
    pub fn fill_percent(&self) -> u64 {
        let usable = (self.page_size as usize - format::PAGE_HEADER_LEN) as u64;
        let capacity = (self.buckets + self.overflow_pages) * usable;
        (self.record_bytes * 100).checked_div(capacity).unwrap_or(0)
    }
}

#[derive(Debug)]
pub struct Database {
    pager: Pager,
    header: Header,
    directory: Vec<u64>,
    sync: bool,
}

/// A bucket's pages as read from disk, with its records gathered together.
struct Chain {
    /// Page numbers and the bytes read from each, bucket page first.
    pages: Vec<(u64, Vec<u8>)>,
    local_depth: u8,
    records: Vec<Record>,
}

/// A record's position in iteration order.
fn order(r: &Record) -> (u64, &[u8]) {
    (r.hash, &r.key)
}

fn corrupt<T>(msg: impl Into<String>) -> Result<T> {
    Err(Error::Corrupt(msg.into()))
}

impl Database {
    /// Creates a new database file. Fails if `path` already exists.
    pub fn create(path: impl AsRef<Path>, options: CreateOptions) -> Result<Self> {
        format::check_page_size(options.page_size)?;
        format::check_max_depth(options.max_depth)?;
        let path = path.as_ref();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path)?;
        Self::initialize(file, options).inspect_err(|_| {
            // Don't leave a half-written file behind.
            let _ = fs::remove_file(path);
        })
    }

    fn initialize(file: File, options: CreateOptions) -> Result<Self> {
        // Page 0: header. Page 1: one-slot directory. Page 2: empty bucket.
        let db = Database {
            pager: Pager::new(file, options.page_size),
            header: Header {
                page_size: options.page_size,
                global_depth: 0,
                max_depth: options.max_depth,
                dir_page: 1,
                dir_pages: 1,
                page_count: 3,
                free_head: 0,
                entry_count: 0,
            },
            directory: vec![2],
            sync: true,
        };
        let mut page0 = db.header.encode().to_vec();
        page0.resize(options.page_size as usize, 0);
        db.pager.write_page(0, &page0)?;
        db.write_directory()?;
        let bucket = Page {
            kind: PageKind::Bucket,
            local_depth: 0,
            next: 0,
            records: Vec::new(),
        };
        db.pager.write_page(2, &bucket.encode(options.page_size)?)?;
        db.pager.sync()?;
        Ok(db)
    }

    /// Opens an existing database for reading and writing.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_file(OpenOptions::new().read(true).write(true).open(path)?)
    }

    /// Opens an existing database for reading only. Mutations will fail.
    pub fn open_read_only(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_file(File::open(path)?)
    }

    fn open_file(file: File) -> Result<Self> {
        let mut buf = [0u8; HEADER_LEN];
        match std::os::unix::fs::FileExt::read_exact_at(&file, &mut buf, 0) {
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                return corrupt("file is too short to be a database");
            }
            result => result?,
        }
        let header = Header::decode(&buf)?;
        let pager = Pager::new(file, header.page_size);

        let expected_len = header.page_count.checked_mul(u64::from(header.page_size));
        let actual_len = pager.file_len()?;
        if expected_len != Some(actual_len) {
            return corrupt(format!(
                "file is {actual_len} bytes, header says {} pages of {}",
                header.page_count, header.page_size
            ));
        }
        let slots = 1usize << header.global_depth;
        let dir_end = header.dir_page.checked_add(header.dir_pages);
        if header.dir_page == 0
            || dir_end.is_none_or(|end| end > header.page_count)
            || header.dir_pages < format::directory_pages(slots, header.page_size)
        {
            return corrupt("directory location is out of range");
        }
        if header.free_head >= header.page_count {
            return corrupt("free list head is out of range");
        }

        let bytes = pager.read_pages(header.dir_page, header.dir_pages)?;
        let directory = format::decode_directory(&bytes, slots)?;
        if let Some(bad) = directory
            .iter()
            .find(|&&p| p == 0 || p >= header.page_count)
        {
            return corrupt(format!("directory points at invalid page {bad}"));
        }
        Ok(Database {
            pager,
            header,
            directory,
            sync: true,
        })
    }

    /// Whether each mutation is flushed to stable storage before returning.
    /// On by default. When off, writes still reach the operating system
    /// immediately but can be lost in a power failure or OS crash; call
    /// [`Database::sync`] to flush them.
    pub fn set_sync(&mut self, sync: bool) {
        self.sync = sync;
    }

    /// Flushes all writes to stable storage.
    pub fn sync(&self) -> Result<()> {
        Ok(self.pager.sync()?)
    }

    pub fn len(&self) -> u64 {
        self.header.entry_count
    }

    pub fn is_empty(&self) -> bool {
        self.header.entry_count == 0
    }

    pub fn global_depth(&self) -> u8 {
        self.header.global_depth
    }

    pub fn page_size(&self) -> u32 {
        self.header.page_size
    }

    /// The options this database was created with.
    pub fn options(&self) -> CreateOptions {
        CreateOptions {
            page_size: self.header.page_size,
            max_depth: self.header.max_depth,
        }
    }

    /// The first page of the bucket that `key` belongs in, whether or not
    /// it is present.
    pub fn bucket_of(&self, key: &[u8]) -> u64 {
        self.bucket_page(key_hash(key))
    }

    fn bucket_page(&self, hash: u64) -> u64 {
        self.directory[prefix(hash, self.header.global_depth.into())]
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let hash = key_hash(key);
        let mut page_no = self.bucket_page(hash);
        let mut kind = PageKind::Bucket;
        for _ in 0..self.header.page_count {
            let page = self.read_chain_page(page_no, kind)?;
            if let Some(r) = page
                .records
                .into_iter()
                .find(|r| r.hash == hash && r.key == key)
            {
                return Ok(Some(r.value));
            }
            if page.next == 0 {
                return Ok(None);
            }
            page_no = page.next;
            kind = PageKind::Overflow;
        }
        corrupt(format!("overflow chain from page {page_no} loops"))
    }

    /// Inserts or replaces `key`. Returns the previous value, if any.
    pub fn put(&mut self, key: &[u8], value: &[u8]) -> Result<Option<Vec<u8>>> {
        let size = key.len() + value.len();
        let max = format::max_payload(self.header.page_size);
        if size > max {
            return Err(Error::TooLarge { size, max });
        }
        let hash = key_hash(key);
        loop {
            let mut chain = self.load_chain(self.bucket_page(hash))?;
            let pos = chain
                .records
                .iter()
                .position(|r| r.hash == hash && r.key == key);
            let old_size = pos.map_or(0, |i| {
                let r = &chain.records[i];
                format::record_size(r.key.len(), r.value.len())
            });
            let new_used = format::used_bytes(&chain.records)
                + format::record_size(key.len(), value.len())
                - old_size;

            if new_used <= self.header.page_size as usize || !self.can_split(&chain, hash) {
                let old = match pos {
                    Some(i) => Some(mem::replace(&mut chain.records[i].value, value.to_vec())),
                    None => {
                        chain.records.push(Record {
                            hash,
                            key: key.to_vec(),
                            value: value.to_vec(),
                        });
                        self.header.entry_count += 1;
                        None
                    }
                };
                self.store_chain(chain.pages, chain.local_depth, chain.records)?;
                self.commit()?;
                return Ok(old);
            }
            self.split(chain, hash)?;
        }
    }

    /// Removes `key`. Returns its value if it was present.
    ///
    /// Buckets are never merged and the directory never shrinks. Overflow
    /// pages that become unnecessary go on the free list for reuse.
    pub fn delete(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let hash = key_hash(key);
        let mut chain = self.load_chain(self.bucket_page(hash))?;
        let Some(pos) = chain
            .records
            .iter()
            .position(|r| r.hash == hash && r.key == key)
        else {
            return Ok(None);
        };
        let removed = chain.records.swap_remove(pos);
        self.store_chain(chain.pages, chain.local_depth, chain.records)?;
        self.header.entry_count -= 1;
        self.commit()?;
        Ok(Some(removed.value))
    }

    /// The first key in iteration order, or `None` if the database is empty.
    ///
    /// Iteration order is by key hash, then by key bytes. Buckets divide the
    /// hash space in directory order, so this is also bucket order, and it
    /// does not depend on how buckets have split or where records sit within
    /// their pages.
    pub fn first_key(&self) -> Result<Option<Vec<u8>>> {
        self.key_after(None)
    }

    /// The key that follows `key` in iteration order, or `None` at the end.
    ///
    /// `key` need not be present. Because the order is fixed, deleting the
    /// current key (or any other) during iteration is safe, and so are
    /// inserts: every key present throughout the iteration is returned
    /// exactly once. A key inserted mid-iteration is returned only if it
    /// sorts after the current position.
    pub fn next_key(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.key_after(Some((key_hash(key), key)))
    }

    /// The smallest key ordered after `after`, or the smallest key overall.
    fn key_after(&self, after: Option<(u64, &[u8])>) -> Result<Option<Vec<u8>>> {
        let dir = &self.directory;
        // Buckets before the one `after` hashes to hold only smaller keys.
        let mut slot = after.map_or(0, |(hash, _)| prefix(hash, self.header.global_depth.into()));
        while let Some(&page) = dir.get(slot) {
            let chain = self.load_chain(page)?;
            let next = chain
                .records
                .into_iter()
                .filter(|r| after.is_none_or(|a| order(r) > a))
                .min_by(|a, b| order(a).cmp(&order(b)));
            if let Some(r) = next {
                return Ok(Some(r.key));
            }
            while dir.get(slot) == Some(&page) {
                slot += 1;
            }
        }
        Ok(None)
    }

    /// Iterates over every `(key, value)` pair in iteration order (see
    /// [`Database::first_key`]), reading one bucket at a time.
    pub fn entries(&self) -> Entries<'_> {
        Entries {
            db: self,
            slot: 0,
            pending: Vec::new().into_iter(),
        }
    }

    /// The first slot of each bucket's run in the directory, with its page.
    fn bucket_runs(&self) -> impl Iterator<Item = (usize, u64)> + '_ {
        self.directory
            .iter()
            .enumerate()
            .filter(|&(slot, &page)| slot == 0 || self.directory[slot - 1] != page)
            .map(|(slot, &page)| (slot, page))
    }

    pub fn stats(&self) -> Result<Stats> {
        let mut buckets = 0;
        let mut overflow_pages = 0;
        let mut record_bytes = 0;
        for (_, page) in self.bucket_runs() {
            let chain = self.load_chain(page)?;
            buckets += 1;
            overflow_pages += chain.pages.len() as u64 - 1;
            record_bytes += (format::used_bytes(&chain.records) - format::PAGE_HEADER_LEN) as u64;
        }
        Ok(Stats {
            page_size: self.header.page_size,
            pages: self.header.page_count,
            global_depth: self.header.global_depth,
            max_depth: self.header.max_depth,
            directory_size: self.directory.len(),
            directory_pages: self.header.dir_pages,
            buckets,
            overflow_pages,
            free_pages: self.free_list()?.len() as u64,
            entries: self.header.entry_count,
            file_size: self.pager.file_len()?,
            record_bytes,
        })
    }

    /// Verifies every structural invariant of the file, returning a
    /// description of the first violation found.
    pub fn check(&self) -> Result<()> {
        let h = &self.header;
        let g = u32::from(h.global_depth);
        let mut owner: Vec<Option<&str>> = vec![None; h.page_count as usize];
        let mut claim = |page: u64, what: &'static str| -> Result<()> {
            match owner.get_mut(page as usize) {
                None => corrupt(format!("{what} page {page} is past the end of the file")),
                Some(Some(prev)) => corrupt(format!("page {page} is both {prev} and {what}")),
                Some(slot) => {
                    *slot = Some(what);
                    Ok(())
                }
            }
        };
        claim(0, "header")?;
        for p in h.dir_page..h.dir_page + h.dir_pages {
            claim(p, "directory")?;
        }
        for p in self.free_list()? {
            claim(p, "free")?;
        }

        let mut total = 0u64;
        for (slot, page) in self.bucket_runs() {
            let chain = self.load_chain(page)?;
            let d = u32::from(chain.local_depth);
            let run = self.directory[slot..]
                .iter()
                .take_while(|&&p| p == page)
                .count();
            if d > g || run != 1 << (g - d) || slot % run != 0 {
                return corrupt(format!(
                    "bucket page {page} (depth {d}) has {run} directory slots from slot {slot}"
                ));
            }
            for (i, (p, _)) in chain.pages.iter().enumerate() {
                claim(*p, if i == 0 { "bucket" } else { "overflow" })?;
            }
            let bucket_prefix = slot >> (g - d);
            for (i, r) in chain.records.iter().enumerate() {
                if r.hash != key_hash(&r.key) {
                    return corrupt(format!("bucket page {page} has a record with a wrong hash"));
                }
                if prefix(r.hash, d) != bucket_prefix {
                    return corrupt(format!(
                        "bucket page {page} holds a key from another bucket"
                    ));
                }
                if chain.records[..i].iter().any(|o| o.key == r.key) {
                    return corrupt(format!("bucket page {page} holds a duplicate key"));
                }
            }
            if chain.pages.len() > 1 && self.can_split_records(&chain, None) {
                return corrupt(format!("bucket page {page} overflows but could split"));
            }
            total += chain.records.len() as u64;
        }
        if total != h.entry_count {
            return corrupt(format!(
                "header counts {} entries, buckets hold {total}",
                h.entry_count
            ));
        }
        if let Some(p) = owner.iter().position(Option::is_none) {
            return corrupt(format!("page {p} is not referenced by anything"));
        }
        Ok(())
    }

    // ---- bucket chains ----

    fn read_page(&self, page: u64) -> Result<Vec<u8>> {
        if page == 0 || page >= self.header.page_count {
            return corrupt(format!("reference to invalid page {page}"));
        }
        Ok(self.pager.read_page(page)?)
    }

    fn read_chain_page(&self, page: u64, kind: PageKind) -> Result<Page> {
        let decoded = Page::decode(&self.read_page(page)?)?;
        if decoded.kind != kind {
            return corrupt(format!(
                "page {page} is a {:?} page, expected {kind:?}",
                decoded.kind
            ));
        }
        Ok(decoded)
    }

    fn load_chain(&self, first: u64) -> Result<Chain> {
        let mut chain = Chain {
            pages: Vec::new(),
            local_depth: 0,
            records: Vec::new(),
        };
        let mut page_no = first;
        loop {
            if chain.pages.len() as u64 >= self.header.page_count {
                return corrupt(format!("overflow chain from page {first} loops"));
            }
            let bytes = self.read_page(page_no)?;
            let page = Page::decode(&bytes)?;
            let expected = if chain.pages.is_empty() {
                chain.local_depth = page.local_depth;
                PageKind::Bucket
            } else {
                PageKind::Overflow
            };
            if page.kind != expected || page.local_depth != chain.local_depth {
                return corrupt(format!("page {page_no} does not belong to bucket {first}"));
            }
            chain.records.extend(page.records);
            chain.pages.push((page_no, bytes));
            if page.next == 0 {
                return Ok(chain);
            }
            page_no = page.next;
        }
    }

    /// Writes `records` as a chain at local depth `depth`, reusing `pages`
    /// (from [`Database::load_chain`], or empty for a new bucket). Allocates
    /// or frees overflow pages as needed and skips pages whose bytes are
    /// unchanged. Returns the bucket page number.
    fn store_chain(
        &mut self,
        pages: Vec<(u64, Vec<u8>)>,
        depth: u8,
        records: Vec<Record>,
    ) -> Result<u64> {
        let groups = format::pack(records, self.header.page_size);
        let mut numbers: Vec<u64> = pages.iter().take(groups.len()).map(|(n, _)| *n).collect();
        while numbers.len() < groups.len() {
            numbers.push(self.allocate_page()?);
        }
        for (i, group) in groups.into_iter().enumerate() {
            let page = Page {
                kind: if i == 0 {
                    PageKind::Bucket
                } else {
                    PageKind::Overflow
                },
                local_depth: depth,
                next: numbers.get(i + 1).copied().unwrap_or(0),
                records: group,
            };
            let bytes = page.encode(self.header.page_size)?;
            if pages.get(i).is_none_or(|(_, old)| *old != bytes) {
                self.pager.write_page(numbers[i], &bytes)?;
            }
        }
        for (n, _) in pages.into_iter().skip(numbers.len()) {
            self.free_page(n)?;
        }
        Ok(numbers[0])
    }

    /// Whether splitting the bucket could separate its records, including
    /// an incoming record with hash `incoming`.
    fn can_split_records(&self, chain: &Chain, incoming: Option<u64>) -> bool {
        if chain.local_depth >= self.header.max_depth {
            return false;
        }
        let mut hashes = chain.records.iter().map(|r| r.hash).chain(incoming);
        match hashes.next() {
            Some(first) => hashes.any(|h| h != first),
            None => false,
        }
    }

    fn can_split(&self, chain: &Chain, hash: u64) -> bool {
        self.can_split_records(chain, Some(hash))
    }

    /// Splits the bucket that `hash` maps to. The new bucket and directory
    /// are written before the old bucket is rewritten without the moved
    /// records.
    fn split(&mut self, chain: Chain, hash: u64) -> Result<()> {
        let d = chain.local_depth;
        if d == self.header.global_depth {
            self.double_directory()?;
        }
        let (moved, kept): (Vec<_>, Vec<_>) = chain
            .records
            .into_iter()
            .partition(|r| bit(r.hash, d.into()));
        let new_page = self.store_chain(Vec::new(), d + 1, moved)?;

        // The bucket's slots form one aligned run; the upper half (next hash
        // bit set) now points at the new bucket.
        let g = u32::from(self.header.global_depth);
        let run = 1usize << (g - u32::from(d));
        let start = prefix(hash, d.into()) * run;
        let upper = start + run / 2..start + run;
        self.directory[upper.clone()].fill(new_page);
        let bytes = format::encode_directory(&self.directory[upper.clone()], 0);
        self.pager
            .write_at(self.header.dir_page, upper.start as u64 * 8, &bytes)?;

        self.store_chain(chain.pages, d + 1, kept)?;
        Ok(())
    }

    // ---- directory ----

    fn write_directory(&self) -> Result<()> {
        let len = (self.header.dir_pages * u64::from(self.header.page_size)) as usize;
        let bytes = format::encode_directory(&self.directory, len);
        Ok(self.pager.write_page(self.header.dir_page, &bytes)?)
    }

    /// Doubles the directory, moving it to the end of the file when it
    /// outgrows its pages. The old pages go on the free list.
    fn double_directory(&mut self) -> Result<()> {
        self.directory = self.directory.iter().flat_map(|&p| [p, p]).collect();
        self.header.global_depth += 1;
        let needed = format::directory_pages(self.directory.len(), self.header.page_size);
        if needed <= self.header.dir_pages {
            return self.write_directory();
        }

        let old = self.header.dir_page..self.header.dir_page + self.header.dir_pages;
        self.header.dir_page = self.header.page_count;
        self.header.dir_pages = needed;
        self.header.page_count += needed;
        self.write_directory()?;
        self.write_header()?;
        for page in old {
            self.free_page(page)?;
        }
        Ok(())
    }

    // ---- page allocation ----

    fn allocate_page(&mut self) -> Result<u64> {
        let head = self.header.free_head;
        if head != 0 {
            let page = self.read_chain_page(head, PageKind::Free)?;
            self.header.free_head = page.next;
            return Ok(head);
        }
        let page = self.header.page_count;
        self.header.page_count += 1;
        Ok(page)
    }

    fn free_page(&mut self, page: u64) -> Result<()> {
        let bytes = Page::free(self.header.free_head).encode(self.header.page_size)?;
        self.pager.write_page(page, &bytes)?;
        self.header.free_head = page;
        Ok(())
    }

    fn free_list(&self) -> Result<Vec<u64>> {
        let mut pages = Vec::new();
        let mut page = self.header.free_head;
        while page != 0 {
            if pages.len() as u64 >= self.header.page_count {
                return corrupt("free list loops");
            }
            pages.push(page);
            page = self.read_chain_page(page, PageKind::Free)?.next;
        }
        Ok(pages)
    }

    // ---- header ----

    fn write_header(&self) -> Result<()> {
        Ok(self.pager.write_at(0, 0, &self.header.encode())?)
    }

    /// Ends a mutation: records the header, then flushes if syncing.
    fn commit(&self) -> Result<()> {
        self.write_header()?;
        if self.sync {
            self.pager.sync()?;
        }
        Ok(())
    }
}

/// Iterator returned by [`Database::entries`].
pub struct Entries<'a> {
    db: &'a Database,
    slot: usize,
    pending: std::vec::IntoIter<Record>,
}

impl Iterator for Entries<'_> {
    type Item = Result<(Vec<u8>, Vec<u8>)>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(r) = self.pending.next() {
                return Some(Ok((r.key, r.value)));
            }
            let dir = &self.db.directory;
            let page = *dir.get(self.slot)?;
            while self.slot < dir.len() && dir[self.slot] == page {
                self.slot += 1;
            }
            match self.db.load_chain(page) {
                Ok(mut chain) => {
                    chain.records.sort_by(|a, b| order(a).cmp(&order(b)));
                    self.pending = chain.records.into_iter();
                }
                Err(e) => {
                    self.slot = dir.len();
                    return Some(Err(e));
                }
            }
        }
    }
}
