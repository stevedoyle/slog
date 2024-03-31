use binread::{io::Cursor, BinReaderExt, NullString};
use clap::Parser;
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

/// tar utility
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Create a new archive containing the specified items.
    #[arg(short, long, action)]
    create: bool,

    /// Like -c, byt the new entries are appended to the archive. Note that this only works on
    /// uncompressed archives stored in regular files. The -f option is required.
    #[arg(short = 'r', long, action)]
    append: bool,

    /// List the archive contents to stdout.
    #[arg(short = 't', long, action)]
    list: bool,

    /// Like -r but new entries are added only if they have a modification date newer than the
    /// corresponding entry in the archive. Note that this only works on uncompressed archives
    /// stored in regular files. The -f option is required.
    #[arg(short, long, action)]
    update: bool,

    /// Extract to disk from the archive. If a file with the same name appears more than once in the
    /// archive, each copy will be extracted, with later copies overwriting (replacing) earlier
    /// copies.
    #[arg(short = 'x', long, action)]
    extract: bool,

    /// Read the archive from or write the arhive to the specified file. The filename can be - for
    /// standard input or standard output.
    #[arg(short, long, value_name = "FILE")]
    file: PathBuf,
}

// tar file format: https://en.wikipedia.org/wiki/Tar_(computing)#File_format
//
// A tar archive consists of a series of file objects, hence the popular term tarball, referencing
// how a tarball collects objects of all kinds that stick to its surface. Each file object includes
// any file data, and is preceded by a 512-byte header record. The file data is written unaltered
// except that its length is rounded up to a multiple of 512 bytes. The original tar implementation
// did not care about the contents of the padding bytes, and left the buffer data unaltered, but
// most modern tar implementations fill the extra space with zeros.[14] The end of an archive is
// marked by at least two consecutive zero-filled records. (The origin of tar's record size appears
// to be the 512-byte disk sectors used in the Version 7 Unix file system.) The final block of an
// archive is padded out to full length with zeros.

#[derive(Debug)]
struct TarFileHeader {
    name: String,
    mode: u64,
    owner: u64,
    group: u64,
    size: u128,
    lastmod: u128,
    checksum: u64,
    link_ind: u8,
    link_name: String,
}

impl TarFileHeader {
    fn new() -> TarFileHeader {
        TarFileHeader {
            name: "".to_string(),
            mode: 0,
            owner: 0,
            group: 0,
            size: 0,
            lastmod: 0,
            checksum: 0,
            link_ind: 0,
            link_name: "".to_string(),
        }
    }
}

fn octal_bytes_to_u64(encoded: &[u8]) -> u64 {
    dbg!(String::from_utf8(encoded.to_vec()).unwrap());
    u64::from_str_radix(&String::from_utf8(encoded.to_vec()).unwrap(), 8).unwrap()
}

fn octal_bytes_to_u128(encoded: &[u8]) -> u128 {
    dbg!(String::from_utf8(encoded.to_vec()).unwrap());
    u128::from_str_radix(&String::from_utf8(encoded.to_vec()).unwrap(), 8).unwrap()
}

fn get_file_header(contents: &[u8]) -> TarFileHeader {
    let mut hdr = TarFileHeader::new();
    hdr.name = Cursor::new(contents[0..100].to_vec())
        .read_be::<NullString>()
        .unwrap()
        .to_string();
    hdr.mode = octal_bytes_to_u64(&contents[100..106]);
    hdr.owner = octal_bytes_to_u64(&contents[108..114]);
    hdr.group = octal_bytes_to_u64(&contents[116..122]);
    hdr.size =
        u128::from_str_radix(&String::from_utf8(contents[124..135].to_vec()).unwrap(), 8).unwrap();
    hdr.lastmod = octal_bytes_to_u128(&contents[136..146]);
    hdr.checksum = octal_bytes_to_u64(&contents[148..154]);
    hdr.link_ind = contents[156];
    hdr.link_name = Cursor::new(contents[157..257].to_vec())
        .read_be::<NullString>()
        .unwrap()
        .to_string();
    hdr
}

fn show_file_info(hdr: &TarFileHeader) {
    println!("{:#?}", hdr);
}

fn list_archive_contents(filename: &Path) {
    let mut f = File::open(filename).expect("cannot read file");
    let mut buf_reader = BufReader::new(f);
    let metadata = f.metadata().expect("unable to read metadata");
    let tarfilelen = metadata.len();
    let blocklen = 512;
    let blocksinfile = tarfilelen / blocklen;
    for _blk in 0..blocksinfile {
        let mut buffer = vec![0; blocklen as usize];
        std::io::Read::read(&mut f, &mut buffer).expect("buffer overflow");
        let hdr = get_file_header(&buffer);
    }

    show_file_info(&hdr);
}

fn main() {
    let args = Args::parse();
    println!("{args:?}");

    if args.list {
        list_archive_contents(&args.file);
    }
}
