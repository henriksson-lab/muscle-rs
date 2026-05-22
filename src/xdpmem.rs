// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct XDPMem {
    pub max_la: uint,
    pub max_lb: uint,
    pub tb_bit: Vec<Vec<byte>>,
    pub tb_bit_row_count: uint,
    pub tb_bit_col_count: uint,
    pub tb_bit_allocated_row_count: uint,
    pub tb_bit_allocated_col_count: uint,
    pub rev_a: Vec<byte>,
    pub rev_b: Vec<byte>,
    pub buffer1: Vec<f32>,
    pub buffer2: Vec<f32>,
} // original: XDPMem (muscle/src/xdpmem.h)
