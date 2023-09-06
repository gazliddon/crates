#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    pub addr: usize,
    pub last_addr: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegionErr {
    SizeIsZero,
    RegionToLargeToFit(usize, usize),
}

impl Region {
    pub fn checked_new(addr: usize, size: usize) -> Result<Self, RegionErr> {
        if size == 0 {
            return Err(RegionErr::SizeIsZero);
        }

        let (addr, last_addr) = calc_addr_last(addr, size);

        if last_addr > 0xffff {
            return Err(RegionErr::RegionToLargeToFit(addr, last_addr));
        }

        Ok(Self::new(addr , size))
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        let (addr, last_addr) = self.to_usize();
        (last_addr - addr) + 1
    }

    pub fn new(addr: usize, mut size: usize) -> Self {
        if size == 0 {
            size = 1;
        }

        let (addr, mut last_addr) = calc_addr_last(addr, size);

        if last_addr >= 0x1_0000 {
            last_addr = 0xffff;
        }

        Self {
            addr,
            last_addr,
        }
    }

    pub fn is_in_region(&self, addr: usize) -> bool {
        self.as_range().contains(&addr)
    }

    fn to_usize(&self) -> (usize, usize) {
        (self.addr , self.last_addr )
    }

    pub fn as_range(&self) -> std::ops::Range<usize> {
        self.addr ..(self.last_addr + 1)
    }
}

fn calc_addr_last(addr: usize, size: usize) -> (usize, usize) {
    (addr , addr + size - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_regions() {
        {
            let start= 0;
            let size = 0x10;

            let mr = Region::checked_new(start, size);
            assert!(mr.is_ok());
            let mr = mr.unwrap();
            assert!(mr.len() == size);

            // let desired = start as usize..=( size - 1);
            // let range = mr.as_range();

            // assert!(range == desired, "wanted {:?} got {:?}", desired, range);

            assert!(mr.is_in_region(start));

            assert!(mr.is_in_region(start + size - 1 ));

            assert!(!mr.is_in_region(start +size ));
        }
    }

    #[test]
    fn size_zero() {
        let mr = Region::checked_new(0, 0);
        assert_eq!(mr, Err(RegionErr::SizeIsZero));
    }
    #[test]
    fn contains() {
        let mr = Region::checked_new(0, 0x1_0000);
        assert!(mr.is_ok());
    }

    #[test]
    fn len() {
        // let len: usize = 0x10;
        // let addr: u16 = 0;
        // let mr = Region::new(addr, len);
        // assert_eq!(mr.len(), len as usize);

        // let desired = addr as usize ..= (addr as usize +len -1 ) as usize;
        //     assert!(
        //         mr.as_range()
        //             == desired
        //     );
    }

    #[test]
    fn too_big() {
        let mr = Region::checked_new(0xffff, 0x2);

        assert!(
            mr == Err(RegionErr::RegionToLargeToFit(0xffff, 0x1_0000)),
            "value was {:04X?}",
            mr
        )
    }
}
