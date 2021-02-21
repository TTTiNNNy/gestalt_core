
#[path = "./desc.rs"]
pub mod desc;
use desc::context::{Context};
use crate::sheduler::{ExecList, Node, List, def_fn};

use crate::context::ContextStatus::Active;

///переключение котнекта сделано через структуру-сингтон
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RtosCell
{
    pub thread_pool: List<Context>,
    pub status_pool: [ContextStatus; desc::TASK_NUM],
    pub active_thread_pool: [usize; desc::CPU_NUM]
}

unsafe impl core::marker::Sync for RtosCell{}

#[derive(Debug, Clone, Copy, Eq)]
pub enum ContextStatus
{
    Active,
    Idle,
    Free,
    Empty
}


impl  PartialEq for ContextStatus{
    fn eq(&self, other: &Self) -> bool {
        self == other
    }

    fn ne(&self, other: &Self) -> bool {
        self != other
    }
}


pub trait ContextMethods
{

    fn save_sw_context(&mut self);
    fn change_context(&mut self);
}


    //
    pub fn get_token(task_fn: fn(), array_for_stack: & mut[u8]) -> usize
    {

        let token;
        unsafe
        {
            token =  RTOS_KERNEL .thread_pool.push(Context
            {
                task_fn: def_fn,
                sp: 1,
                rons: [0;8],
            });
            let mut stk_addr =array_for_stack.get(array_for_stack.len()-1).unwrap()  as *const u8 as usize;
            while(stk_addr % 4) != 0 {stk_addr-=1;}
            stk_addr-=64;
            RTOS_KERNEL.status_pool[token] = ContextStatus::Empty;
            RTOS_KERNEL.thread_pool.el_crate[token].elem.task_fn = task_fn;
            RTOS_KERNEL.thread_pool.el_crate[token].elem.sp = stk_addr;
        }
        token
    }
    fn free_token(numb: usize)
    {
        unsafe
        {
            RTOS_KERNEL.thread_pool.el_crate[numb].elem.sp = 0;
            RTOS_KERNEL.status_pool[numb] = ContextStatus::Free;
        }
    }

    fn gluing(token_1: usize, token_2: usize)
    {
        unsafe { RTOS_KERNEL.thread_pool.el_crate[token_1].next = token_2; }
    }


pub fn rtos_start()
{
    unsafe
        {
            if RTOS_KERNEL.thread_pool.el_crate[2].elem.sp != 0
            {
                RTOS_KERNEL.status_pool[2] = Active;
                asm!(
                "msr control, r0",
                "mov sp,    {1}",
                "mov pc,    {0}",
                in(reg) RTOS_KERNEL.thread_pool.el_crate[RTOS_KERNEL.active_thread_pool[0]].elem.task_fn,
                in(reg) RTOS_KERNEL.thread_pool.el_crate[RTOS_KERNEL.active_thread_pool[0]].elem.sp,
                in("r0") 2
                );
            }

        }
}

pub(crate) static mut RTOS_KERNEL:self::RtosCell = self::RtosCell
{
    thread_pool: List {
    empty: Context
    {
        task_fn: def_fn,
        sp: 0,
        rons: [0,0,0,0,0,0,0,0],
    },
    el_crate:
    [Node
    {
        elem: Context
        {
            task_fn: def_fn,
            sp: 0,
            rons: [0,0,0,0,0,0,0,0],

        },
        next: 0 ,
        prev: 0 ,
    }; 10],
},
    status_pool: [ContextStatus::Free; desc::TASK_NUM],
    active_thread_pool: [2; desc::CPU_NUM]
};
