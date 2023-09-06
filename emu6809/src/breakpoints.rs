
use std::collections::HashMap;

use crate::mem::{Region, RegionErr};

#[derive(Clone, Debug, PartialEq, PartialOrd, Copy)]
pub enum BreakPointTypes {
    READ,
    WRITE,
    EXEC,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Copy)]
pub struct BreakPoint {
    addr: usize,
    bp_type: BreakPointTypes,
    active: bool,
    id: usize,
}

impl BreakPoint {
    /// Describes a breakpoint constructs defaulted to active
    pub fn new(bp_type: BreakPointTypes, addr: usize, id: usize) -> BreakPoint {
        BreakPoint {
            bp_type,
            addr,
            active: true,
            id,
        }
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn toggle_active(&mut self) {
        self.active = !self.active;
    }
}

// usize def as zero
#[derive(Default, Clone, Debug)]
pub struct BreakPoints {
    break_points: std::collections::HashMap<usize, BreakPoint>,
    mem_to_bp: HashMap<usize, usize>,
    id: usize,
}

impl BreakPoints {
    pub fn new() -> BreakPoints {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.break_points.len()
    }

    pub fn has_any_breakpoint(&self, addr: usize) -> bool {
        let bp = self.get_breakpoints(addr, 1);
        bp.iter().filter(|b| b.addr == addr).count() > 0
    }

    pub fn has_breakpoint(&self, addr: usize, bp_type: BreakPointTypes) -> bool {
        self.find_breakpoint(addr, bp_type).is_some()
    }

    fn find_breakpoint(&self, addr: usize, bp_type: BreakPointTypes) -> Option<&BreakPoint> {
        self.break_points
            .values()
            .find(|bp| bp.addr == addr && bp.bp_type == bp_type)
    }

    pub fn add(&mut self, addr: usize, bp_type: BreakPointTypes) -> Option<usize> {
        if !self.has_breakpoint(addr, bp_type) {
            let ret = self.id;
            let bp = BreakPoint::new(bp_type, addr, ret);
            self.id += 1;
            self.break_points.insert(ret, bp);
            Some(ret)
        } else {
            None
        }
    }
    pub fn find_breakpoint_id(&self, addr: usize, bp_type: BreakPointTypes) -> Option<usize> {
        self.find_breakpoint(addr, bp_type)
            .map(|bp| bp.id)
    }

    pub fn remove_by_id(&mut self, id: usize) {
        self.break_points.remove(&id);
    }

    pub fn remove(&mut self, addr: usize, bp_type: BreakPointTypes) {
        if let Some(id) = self.find_breakpoint_id(addr, bp_type) {
            self.break_points.remove(&id);
        }
    }

    fn get_range(addr: usize, range: usize) -> Result<Region, RegionErr> {
        Region::checked_new(addr, range)
    }

    pub fn get_breakpoints(&self, addr: usize, range: usize) -> Vec<&BreakPoint> {
        if let Ok(r) = Self::get_range(addr, range) {
            self.break_points
                .values()
                .filter(|bp| r.is_in_region(bp.addr))
                .collect()
        } else {
            vec![]
        }
    }

    pub fn get_breakpoints_mut(&mut self, addr: usize, range: usize) -> Vec<&mut BreakPoint> {
        if let Ok(r) = Self::get_range(addr, range) {
            self.break_points
                .values_mut()
                .filter(|bp| r.is_in_region(bp.addr))
                .collect()
        } else {
            vec![]
        }
    }

    pub fn remove_all_at_addr(&mut self, addr: usize) {
        let v: Vec<usize> = self.get_breakpoints(addr, 1).iter().map(|b| b.id).collect();

        for id in v {
            self.break_points.remove(&id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: write tests
    #[test]
    fn count() {
        let addr = 0;

        let mut bp = BreakPoints::new();
        assert_eq!(bp.len(), 0);

        bp.add(addr as usize, BreakPointTypes::READ);

        let matched = bp.get_breakpoints(addr, 1);

        assert_eq!(matched.len(), 1);

        assert!(bp.has_breakpoint(addr, BreakPointTypes::READ));
        assert_eq!(bp.len(), 1);

        bp.add(addr, BreakPointTypes::READ);

        assert_eq!(bp.len(), 1);
        bp.add(addr, BreakPointTypes::WRITE);
        assert_eq!(bp.len(), 2);
        bp.remove(addr, BreakPointTypes::READ);
        assert_eq!(bp.len(), 1);
    }
}
