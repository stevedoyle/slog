
// Use the newtype pattern to represent a byte as a collection of flags.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flags(u8);

/// In this example, Flags is a single byte that has the following fields:
/// - bit 0: S. Some made up flag called 'S'
/// - bit 1: D. Some made up flag called 'D'
/// - bits 2-5: A 4 bit field that contains a version identifier
/// - bit 7: A reserved value that must be set to 1
///
/// We want to treat this as a type that has default values and can be
/// manipulated and serialized. The individual bit fields can also be accessed
/// via getters and setters.
impl Flags {
    pub fn new(value: u8) -> Flags {
        Flags { 0: value, }
    }

    pub fn s(&self) -> bool {
        self.0 & 0x01 == 1
    }

    pub fn set_s<'a>(&'a mut self, val: bool) -> &'a mut Flags {
        match val {
            true  => self.0 |= 0x01,
            false => self.0 &= 0xFE,
        }
        self
    }

    pub fn d(&self) -> bool {
        (self.0 >> 1) & 0x01 == 1
    }

    pub fn set_d<'a>(&'a mut self, val: bool) -> &'a mut Flags {
        match val {
            true => self.0 |= 0x02,
            false => self.0 &= 0xFD,
        }
        self
    }

    pub fn version(&self) -> u8 {
        (self.0 >> 2) & 0x0F
    }

    pub fn set_version(&mut self, val: u8) -> &mut Flags {
        self.0 |= (val << 2) & 0xC3;
        self
    }

    pub fn build(&mut self) -> Flags {
        *self
    }
}

impl Default for Flags {
    fn default() -> Self {
        Flags { 0: 0x80, }
    }
}

#[cfg(test)]
mod tests {
    use crate::Flags;

    #[test]
    fn check_default() {
        assert_eq!(Flags::default().0, 0x80);
    }

    #[test]
    fn check_set_s() {
        let mut flags = Flags { 0: 0 };
        flags.set_s(true);
        assert_eq!(flags.0, 1);
        flags.set_s(false);
        assert_eq!(flags.0, 0);
    }

    #[test]
    fn check_set_d() {
        let mut flags = Flags { 0: 0 };
        flags.set_d(true);
        assert_eq!(flags.0, 2);
        flags.set_d(false);
        assert_eq!(flags.0, 0);
    }

    #[test]
    fn check_set_s_and_d() {
        let mut flags = Flags { 0: 0 };
        flags.set_s(true).set_d(true);
        assert_eq!(flags.0, 3);
        flags.set_s(true).set_d(false);
        assert_eq!(flags.0, 1);
        flags.set_s(false).set_d(true);
        assert_eq!(flags.0, 2);
        flags.set_s(false).set_d(false);
        assert_eq!(flags.0, 0);
    }

    #[test]
    fn check_builder_with_default() {
        let flags = Flags::default()
            .set_s(true)
            .set_d(true)
            .build();
        assert_eq!(flags, Flags(0x83));
    }

}
