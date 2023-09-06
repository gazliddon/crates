use super::mem::{ MemoryIO };
use super::cpu::Clock;

use sha1::Sha1;

use std::cell::RefCell;
use std::rc::Rc;

////////////////////////////////////////////////////////////////////////////////
trait Bits {
    fn get_bit(&self, bit : usize) -> bool;
}

impl Bits for u8 {
    fn get_bit(&self, bit : usize) -> bool{
        if bit < 8 {
            (*self & 1 << bit) != 0
        } else {
            false
        }
    }
}


// http://www.playvectrex.com/designit/chrissalo/via3.htm

// VIA_port_b      EQU     $D000   ;VIA port B data I/O register
// *       0 sample/hold (0=enable  mux 1=disable mux)
// *       1 mux sel 0
// *       2 mux sel 1
// *       3 sound BC1
// *       4 sound BDIR
// *       5 comparator input
// *       6 external device (slot pin 35) initialized to input
// *       7 /RAMP
// VIA_port_a      EQU     $D001   ;VIA port A data I/O register (handshaking)
// VIA_DDR_b       EQU     $D002   ;VIA port B data direction register (0=input 1=output)
// VIA_DDR_a       EQU     $D003   ;VIA port A data direction register (0=input 1=output)
// VIA_t1_cnt_lo   EQU     $D004   ;VIA timer 1 count register lo (scale factor)
// VIA_t1_cnt_hi   EQU     $D005   ;VIA timer 1 count register hi
// VIA_t1_lch_lo   EQU     $D006   ;VIA timer 1 latch register lo
// VIA_t1_lch_hi   EQU     $D007   ;VIA timer 1 latch register hi
// VIA_t2_lo       EQU     $D008   ;VIA timer 2 count/latch register lo (refresh)
// VIA_t2_hi       EQU     $D009   ;VIA timer 2 count/latch register hi
// VIA_shift_reg   EQU     $D00A   ;VIA shift register
// VIA_aux_cntl    EQU     $D00B   ;VIA auxiliary control register
// *       0 PA latch enable
// *       1 PB latch enable
// *       2 \                     110=output to CB2 under control of phase 2 clock
// *       3  > shift register control     (110 is the only mode used by the Vectrex ROM)
// *       4 /
// *       5 0=t2 one shot                 1=t2 free running
// *       6 0=t1 one shot                 1=t1 free running
// *       7 0=t1 disable PB7 output       1=t1 enable PB7 output
// VIA_cntl        EQU     $D00C   ;VIA control register
// *       0 CA1 control     CA1 -> SW7    0=IRQ on low 1=IRQ on high
// *       1 \
// *       2  > CA2 control  CA2 -> /ZERO  110=low 111=high
// *       3 /
// *       4 CB1 control     CB1 -> NC     0=IRQ on low 1=IRQ on high
// *       5 \
// *       6  > CB2 control  CB2 -> /BLANK 110=low 111=high
// *       7 /
// VIA_int_flags   EQU     $D00D   ;VIA interrupt flags register
// *               bit                             cleared by
// *       0 CA2 interrupt flag            reading or writing port A I/O
// *       1 CA1 interrupt flag            reading or writing port A I/O
// *       2 shift register interrupt flag reading or writing shift register
// *       3 CB2 interrupt flag            reading or writing port B I/O
// *       4 CB1 interrupt flag            reading or writing port A I/O
// *       5 timer 2 interrupt flag        read t2 low or write t2 high
// *       6 timer 1 interrupt flag        read t1 count low or write t1 high
// *       7 IRQ status flag               write logic 0 to IER or IFR bit
// VIA_int_enable  EQU     $D00E   ;VIA interrupt enable register
// *       0 CA2 interrupt enable
// *       1 CA1 interrupt enable
// *       2 shift register interrupt enable
// *       3 CB2 interrupt enable
// *       4 CB1 interrupt enable
// *       5 timer 2 interrupt enable
// *       6 timer 1 interrupt enable
// *       7 IER set/clear control
// VIA_port_a_nohs EQU     $D00F   ;VIA port A data I/O register (no handshaking)

#[derive(Debug, Clone)]
#[repr(u16)]
pub enum Reg {

    PortB      = 0x0,
    PortA      = 0x1,

    DdrB       = 0x2,
    DdrA       = 0x3,

    T1CntL     = 0x4,
    T1CntH     = 0x5,

    T1LatchLo  = 0x6,
    T1LatchHi  = 0x7,

    T2Lo       = 0x8,
    T2Hi       = 0x9,

    ShiftReg   = 0xa,

    AuxCntl    = 0xb,

    Cntl       = 0xc,
    IntFlags   = 0xd,
    IntEnable  = 0xe,
    PortANhs   = 0xf,
}



////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Default)]
pub struct Port {
    pub bits : u8,
    pub ddr : u8,
    pub latch_enabled : bool,
}

impl Port {
    pub fn new(bits : u8, _ddr : u8) -> Port {
        Port { bits, ddr : 0, latch_enabled : false }
    }

    fn set_ddr(&mut self, val : u8) { self.ddr = val; }

    fn set_val(&mut self, val : u8) { 
        self.bits = val;
    }

    fn get_ddr(&self) -> u8  { self.ddr }
    fn get_val(&self) -> u8 { self.bits }
    fn write_port(&mut self, val : u8) {
        let ddr = self.get_ddr();
        let p = self.get_val(); 

        self.set_val(( p & !ddr ) | val & ddr);
    }

    fn read_port(&self) -> u8 {
        self.get_val() & !self.get_ddr()
    }
}


////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Default)]
struct Timer {
    pub counter : u16,
    pub latch : u16,
    pub free_run : bool,
    pub int_flag : bool,
    pub timer_1 : bool,
}

impl Timer {

    pub fn new(timer_1 : bool) -> Timer {
        Timer {counter : 0, latch : 0, free_run : false, int_flag : false, timer_1}
    }

    pub fn write_lo(&mut self, val : u8) {
        self.write_latch_lo(val)
    }

    pub fn write_hi(&mut self, val : u8) {
        let data = ( self.counter & 0xff ) | u16::from(val) << 8;
        self.latch = data;
        self.counter = data;
        self.reset_int_flag()
    }

    pub fn write_latch_lo(&mut self, val : u8) {
        self.latch = ( self.latch & 0xff00 ) | u16::from(val );
    }

    pub fn write_latch_hi(&mut self, val : u8) {
        self.latch = ( self.latch & 0xff00 ) | u16::from(val );
        self.reset_int_flag()
    }

    pub fn reset_int_flag(&mut self ) {
        self.int_flag = false;
    }

    pub fn read_lo(&mut self) -> u8 {
        self.reset_int_flag();
        self.counter as u8
    }

    pub fn read_hi(&self) -> u8 {
        (self.counter >> 8)as u8
    }
    pub fn read_latch_lo(&self) -> u8 {
        self.latch as u8
    }

    pub fn read_latch_hi(&self) -> u8 {
        (self.latch >> 8)as u8
    }
}

////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Default)]
pub struct M6522<C : Clock> {
    start : u16,
    size :u16,
    last_byte : u16,
    name : String,
    rc_clock : Rc<RefCell<C>>,
    dirty_flag : bool,

    timer_1 : Timer,
    timer_2 : Timer,

    port_b : Port,
    port_a : Port,

    aux_cntl : u8,
    cntl : u8,

    shift_reg : u8,
}

#[derive(Debug, Clone)]
pub enum SoundReg {
    TBD,
}
#[derive(Debug, Clone)]
pub enum MuxDest {
    Disabled,
    XAxis,
    YAxis,
    XYAxisIntegrator,
    ZAxis,
    SoundChip,
}

impl <C : Clock> M6522 <C> {


    pub fn ramp(&self) -> bool { self.port_b.bits.get_bit(7) }
    pub fn comparator(&self) -> bool { self.port_b.bits.get_bit(5) }
    pub fn sample_hold(&self) -> bool { self.port_b.bits.get_bit(1)  }

    pub fn sound(&self) -> SoundReg {
        match (self.port_b.bits >> 3) & 3 {
            _ => SoundReg::TBD,
        }
    }

    pub fn get_mux_dest(&self) -> MuxDest {
        if self.sample_hold() {
            MuxDest::XAxis
        } else {

            match (self.port_b.bits >> 1) & 3  {
                0 => MuxDest::YAxis,
                1 => MuxDest::XYAxisIntegrator,
                2 => MuxDest::ZAxis,
                3 => MuxDest::SoundChip,
                _ => panic!("really?")
            }
        }
    }


    pub fn port_b_report(&self) {

        let rtext = if self.ramp() {
            "true : gun on" 
        } 
        else { 
            "false : gun off"
        };

        println!("PortB setup");
        println!("MuxDest    : {:?}", self.get_mux_dest());
        println!("SOUND      : {:?}", self.sound());
        println!("COMPARATOR : {}", self.comparator());
        println!("RAMP       : {} ", rtext);
    }
}

impl <C : Clock> M6522 <C> {

    pub fn clear_dirty(&mut self) {
        self.dirty_flag = false;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty_flag
    }

    fn set_dirty(&mut self) {
        self.dirty_flag = true;
    }

    pub fn new(start : u16, size : u16, rc_clock : &Rc<RefCell<C>>) -> Self {

        let last_byte = (u32::from(size) + u32::from(start)) - 1;

        assert!(last_byte < 0x1_0000);

        Self {
            start,
            size,
            last_byte : last_byte as u16,
            name : format!("6522 : {:04x} {:04x}", start, size),
            dirty_flag : false,
            rc_clock : rc_clock.clone(),
            port_b : Port::new(0,0),
            port_a : Port::new(0,0),
            timer_1 : Timer::new(true),
            timer_2 : Timer::new(false),
            aux_cntl : 0,
            cntl : 0,
            shift_reg : 0,
        }
    }

    pub fn get_reg(&self, addr : u16) -> (Reg, usize) {

        let reg_num = (addr - self.start) & 0xf;

        use self::Reg::*;

        let r = match reg_num {
            0x0 => PortB,
            0x1 => PortA,

            0x2 => DdrB ,
            0x3 => DdrA ,

            0x4 => T1CntL,
            0x5 => T1CntH,

            0x6 => T1LatchLo,
            0x7 => T1LatchHi,

            0x8 => T2Lo,
            0x9 => T2Hi,

            0xa => ShiftReg,

            0xb => AuxCntl,

            0xc => Cntl,
            0xd => IntFlags,
            0xe => IntEnable,
            0xf => PortANhs,
            _ => panic!("no"),

        };

        (r, reg_num as usize)
    }

    ////////////////////////////////////////////////////////////////////////////////
    fn write_cntl(&mut self, data : u8) {
        self.cntl = data;

        // let ca1_irq_on_high = data.get_bit(0);
        // let ca1_cntl = (data >> 1) & 7;

        // let ca2_irq_on_high = data.get_bit(4);
        // let ca2_cntl = (data >> 5) & 3;
    }

    fn get_ca1_irq_on_high(&self) -> bool {
        self.cntl.get_bit(0)

    } 
    fn get_ca2_cntl(&self) -> u8 {
        (self.cntl >> 1) & 7
    }

    fn get_cb1_irq_on_high(&self)  -> bool {

        self.cntl.get_bit(4)
    }
    fn get_cb2_cntl(&self)  -> u8 {
        (self.cntl >> 5) & 3
    }

    fn cntl_report(&self) {
        println!("cntl");
        println!("ca1 irq hi   : {}", self.get_ca1_irq_on_high());
        println!("ca2 cntl     : {}", self.get_ca2_cntl());
        println!("cb1 irq hi   : {}", self.get_cb1_irq_on_high());
        println!("cb2 cntl     : {}", self.get_cb2_cntl());
        println!("bits         : %{:08b}", self.cntl);
    }
    ////////////////////////////////////////////////////////////////////////////////

    fn write_aux_cntl(&mut self, data : u8) {
        self.aux_cntl = data;
        let t1_free_run = data.get_bit(5);
        let t2_free_run = data.get_bit(6);
        self.timer_1.free_run = t1_free_run;
        self.timer_2.free_run = t2_free_run;
    }

    fn aux_cntl_report(&self) {
        println!("auxcntl");
        println!("PA latch     : {}", self.get_port_a_latch());
        println!("PB latch     : {}", self.get_port_b_latch());
        println!("SR control   : {}", self.get_sr_control());
        println!("T1 free run  : {}", self.timer_1.free_run);
        println!("T2 free run  : {}", self.timer_2.free_run);
        println!("p7 enable    : {}", self.get_t1_p7_enable());
        println!("bits         : %{:08b}", self.aux_cntl);
    }

    pub fn get_t1_p7_enable(&self) -> bool {
        self.aux_cntl.get_bit(7)
    }

    pub fn get_sr_control(&self) -> u8 {
        (self.aux_cntl >> 2) & 0b111
    }

    pub fn get_port_a_latch(&self) -> bool {
        self.aux_cntl.get_bit(0)
    }

    pub fn get_port_b_latch(&self) -> bool {
        self.aux_cntl.get_bit(1)
    }
}


////////////////////////////////////////////////////////////////////////////////


impl<C : Clock> MemoryIO for M6522<C> {

    fn get_range(&self) -> (u16, u16) {
        (self.start, self.last_byte)
    }

    fn update_sha1(&self, _digest : &mut Sha1) {
        panic!();
    }

    fn upload(&mut self, _addr : u16, _data : &[u8]) {
        panic!("tbd")
    }

    fn get_name(&self) -> String {
        "via".to_string()
    }


    // http://archive.6502.org/datasheets/synertek_sy6522.pdf

    fn load_byte(&mut self, addr:u16) -> u8 {
        self.set_dirty();
        let (reg, _) = self.get_reg(addr);
        println!("R  0x{:04X} {:?}",addr, reg);

        use self::Reg::*;

        match reg {
            DdrA        => self.port_a.get_ddr() ,
            PortA       => self.port_a.read_port(),
            DdrB        => self.port_b.get_ddr() ,
            PortB       => self.port_b.read_port(),
            AuxCntl     => self.aux_cntl ,
            Cntl        => self.cntl,
            T1CntL      => self.timer_1.read_lo() ,
            T1CntH      => self.timer_1.read_hi() ,
            T1LatchLo   => self.timer_1.read_latch_lo(),
            T1LatchHi   => self.timer_1.read_latch_hi(),
            T2Lo        => self.timer_2.read_lo(),
            T2Hi        => self.timer_2.read_hi(),

            _ => panic!("Unhandled read for {:?}", reg),


            // ShiftReg    => {10} ,
            // IntFlags    => {13} ,
            // IntEnable   => {14} ,
            // PortANhs    => {15} ,
        }
    }

    fn store_word(&mut self, addr:u16, val:u16) {
        self.store_byte(addr, (val >> 8) as u8);
        self.store_byte(addr.wrapping_add(1), val as u8);
    }

    fn store_byte(&mut self, addr:u16, val:u8) {
        self.set_dirty();
        let (reg, _) = self.get_reg(addr);

        let reg_str = format!("{:?}", reg);
        println!("W  0x{:04X} {:10} : 0x{:02x} 0b{:08b}",addr, reg_str, val, val);

        use self::Reg::*;

        match reg {
            DdrA         => self.port_a.set_ddr(val),
            PortA        => {
                self.port_a.write_port(val);
            },

            DdrB         => self.port_b.set_ddr(val),

            PortB        => {
                self.port_b.write_port(val);
                self.port_b_report()
            }

            AuxCntl      => {
                self.write_aux_cntl(val);
                // self.aux_cntl_report();
            },

            T1CntL       => self.timer_1.write_lo(val),
            T1CntH       => self.timer_1.write_hi(val),

            Cntl             => {
                self.write_cntl(val);
                // self.cntl_report()
            },

            ShiftReg     => self.shift_reg = val,

            T1LatchLo    => self.timer_1.write_latch_lo(val),
            T1LatchHi    => self.timer_1.write_latch_hi(val),
            T2Lo         => self.timer_2.write_lo(val),
            T2Hi         => self.timer_2.write_hi(val),

            _ => panic!("Unhandled write of %{:08b} to {:?}", val, reg)

                // PortA        => () ,

                // T2CntL       => () ,
                // T2CntH       => () ,

                // T2Lo         => () ,
                // T2Hi         => () ,



                // IntFlags     => () ,
                // PortANhs     => () ,
        };

    }
}
