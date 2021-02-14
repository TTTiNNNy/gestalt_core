#![no_std]
#![no_main]

#[allow(unused_imports)]

use panic_halt;
use cortex_m::{Peripherals};
use cortex_m_rt::entry;
use gestalt_core::context::desc::{save_sw_context,change_context};
use gestalt_core::sheduler::{new, ExecList};
use gestalt_core::sheduler::ActiveElSetting::NEXT;
use gestalt_core::context::{rtos_start, get_token};

#[no_mangle]
fn SysTick()
{
    save_sw_context();
//do_something...
    change_context(NEXT);
}

static mut STACK1:[u8;1024] = [0;1024];
static mut STACK2:[u8;1024] = [0;1024];

fn blink_abstract_led1()
{
    let mut a = 0;
    while a < 100 {
        a+=1;
    }
}
fn blink_abstract_led2()
{
    let mut b = 100;
    while b > 1 {
        b-=1;
    }
}

fn blink_abstract_led3()
{
    let mut c = 0;
    while c < 100 {
        c+=1;
    }
}

fn blink_abstract_led4()
{
    let mut d = 100;
    while d < 1 {
        d-=1;
    }
}

fn task1()
{

    let mut pool =new();
    pool.push(blink_abstract_led1);
    pool.push(blink_abstract_led2);
    loop{ pool.exec_all(); }

}

fn task2()
{

    let mut pool =new();
    pool.push(blink_abstract_led3);
    pool.push(blink_abstract_led4);
    loop { pool.exec_all(); }

}

#[entry]
fn main() -> !
{
    let mut per = Peripherals::take().unwrap();
    per.SYST.set_reload(32_000);
    per.SYST.enable_counter();
    per.SYST.enable_interrupt();


    let task1_token;
    let task2_token;

    unsafe
    {
        task1_token = get_token(task1, STACK1.as_mut());
        task2_token = get_token(task2, STACK2.as_mut());
    }

    rtos_start();
    loop
    {
    }
}
