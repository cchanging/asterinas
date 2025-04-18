// SPDX-License-Identifier: MPL-2.0

//! The console I/O.

use alloc::{fmt, sync::Arc, vec::Vec};
use core::fmt::Write;

use log::debug;
use spin::Once;

use super::{
    device::serial::SerialPort,
    kernel::{self, IO_APIC},
};
use crate::{
    io::reserve_io_port_range,
    sync::SpinLock,
    trap::{IrqLine, TrapFrame},
};

/// Prints the formatted arguments to the standard output using the serial port.
#[inline]
pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

/// The callback function for console input.
pub type InputCallback = dyn Fn(u8) + Send + Sync + 'static;

/// Registers a callback function to be called when there is console input.
pub fn register_console_input_callback(f: &'static InputCallback) {
    SERIAL_INPUT_CALLBACKS
        .disable_irq()
        .lock()
        .push(Arc::new(f));
}

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &c in s.as_bytes() {
            send(c);
        }
        Ok(())
    }
}

bitflags::bitflags! {
  struct LineSts: u8 {
    const INPUT_FULL = 1;
    const OUTPUT_EMPTY = 1 << 5;
  }
}

static CONSOLE_COM1_PORT: SerialPort = unsafe { SerialPort::new(0x3F8) };
reserve_io_port_range!(0x3F8..0x400);

static CONSOLE_IRQ_CALLBACK: Once<SpinLock<IrqLine>> = Once::new();
static SERIAL_INPUT_CALLBACKS: SpinLock<Vec<Arc<InputCallback>>> = SpinLock::new(Vec::new());

/// Initializes the serial port.
pub(crate) fn init() {
    CONSOLE_COM1_PORT.init();
}

pub(crate) fn callback_init() {
    let irq = if !IO_APIC.is_completed() {
        let mut irq = crate::arch::x86::kernel::pic::allocate_irq(4).unwrap();
        irq.on_active(|trapframe: &TrapFrame| {
            handle_serial_input(trapframe);
            kernel::pic::ack();
        });
        irq
    } else {
        let mut irq = IrqLine::alloc().unwrap();
        let mut io_apic = IO_APIC.get().unwrap().first().unwrap().lock();
        io_apic.enable(4, irq.clone()).unwrap();
        irq.on_active(handle_serial_input);
        irq
    };

    CONSOLE_IRQ_CALLBACK.call_once(|| SpinLock::new(irq));
}

fn handle_serial_input(_trap_frame: &TrapFrame) {
    let lock = if let Some(lock) = SERIAL_INPUT_CALLBACKS.try_lock() {
        lock
    } else {
        return;
    };
    let received_char = receive_char().unwrap();
    debug!("receive char = {:?}", received_char);
    for callback in lock.iter() {
        callback(received_char);
    }
}

fn line_sts() -> LineSts {
    LineSts::from_bits_truncate(CONSOLE_COM1_PORT.line_status())
}

/// Sends a byte on the serial port.
pub fn send(data: u8) {
    match data {
        8 | 0x7F => {
            while !line_sts().contains(LineSts::OUTPUT_EMPTY) {}
            CONSOLE_COM1_PORT.send(8);
            while !line_sts().contains(LineSts::OUTPUT_EMPTY) {}
            CONSOLE_COM1_PORT.send(b' ');
            while !line_sts().contains(LineSts::OUTPUT_EMPTY) {}
            CONSOLE_COM1_PORT.send(8);
        }
        _ => {
            while !line_sts().contains(LineSts::OUTPUT_EMPTY) {}
            CONSOLE_COM1_PORT.send(data);
        }
    }
}

/// Receives a byte on the serial port. non-blocking
fn receive_char() -> Option<u8> {
    if line_sts().contains(LineSts::INPUT_FULL) {
        Some(CONSOLE_COM1_PORT.recv())
    } else {
        None
    }
}
