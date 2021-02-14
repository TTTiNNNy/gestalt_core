use crate::context::{RTOS_KERNEL, ContextStatus};
use crate::sheduler::ActiveElSetting;

static mut kernel_stack: [u8;512] = [0;512];


/// структура хранения всех регистров, которые не сохраняет в стек сам камень
#[derive(Debug, Clone, Copy, Eq)]
pub struct Context
{
    pub task_fn: fn(),
    pub sp: usize,
    pub rons: [usize;8],
}
/// определение поведения сравнения структуры хранения регистров, которые ядро не сохранило в стек автоматически
impl PartialEq for Context
{
    fn eq(&self, other: &Self) -> bool {
        self.sp == other.sp
    }

    fn ne(&self, other: &Self) -> bool {
        self.sp != other.sp
    }
}

///Первая функция, которая должна быть исполнена в прерывании, иначе код раннее может изменить r4-r11.
///Сохраняет регистры и стек в нужнуй стуктуру
#[inline(always)]
pub fn save_sw_context ()
{
    unsafe
        {
            asm!
            (
            "mrs {0},   psp",
            "stm r0, {{r4-r11}} ",
            out(reg) RTOS_KERNEL.thread_pool.el_crate[RTOS_KERNEL.active_thread_pool[0]].elem.sp,
            in("r0") RTOS_KERNEL.thread_pool.el_crate[RTOS_KERNEL.active_thread_pool[0]].elem.rons.as_mut_ptr(),
            );
        }
}

///Вызывается исключительно тогда, когда следующий контект будет работать впервые
/// т.е. у него еще нет сохраненные регистров в стеке.
/// Это меняет поведение смены контекста на "грубый" переход
fn prepare_task_stack_once()
{

    unsafe
        {
            let mut stk_addr =kernel_stack.get(kernel_stack.len()-1).unwrap()  as *const u8 as usize;
            while(stk_addr % 4) != 0 {stk_addr-=1;}
            stk_addr-=64;
            asm!
            (
            "mov sp, {2}",          // Из-за грубого прыжка в новый контекст, каждый раз стек будет увеличиватся, пока память не закончится.
            "msr control, r0",      // Принудительно обнуляем стек перед прыжком.
            "mov sp, {1}",
            "mov pc, {0}",
            in(reg) RTOS_KERNEL.thread_pool.el_crate[RTOS_KERNEL.active_thread_pool[0]].elem.task_fn,
            in(reg) RTOS_KERNEL.thread_pool.el_crate[RTOS_KERNEL.active_thread_pool[0]].elem.sp,
            in(reg) stk_addr, // 2 - второй бит, отвечащий за стек. котоый будет использоваться в данный момент
            in("r0") 2,

            );
        }

}

/// Функция, которая может быть вызвана в любое время. Переключает контекст
#[inline(never)]
pub fn change_context(el: ActiveElSetting)
{

    unsafe
        {

/// В зависимости от значения перечисления, поступившего в функцию, выбираем следующий контекст.

            let mut next_el;
            match el
            {
                ActiveElSetting::PREV => next_el = RTOS_KERNEL.thread_pool.el_crate[RTOS_KERNEL.active_thread_pool[0]].prev,
                ActiveElSetting::NEXT => next_el = RTOS_KERNEL.thread_pool.el_crate[RTOS_KERNEL.active_thread_pool[0]].next,
                ActiveElSetting::ARBITRARY(num) => next_el = num,
            }
            if(next_el) == 0 {next_el = RTOS_KERNEL.thread_pool.el_crate[next_el].next;}
            let mut stk_addr =kernel_stack.get(kernel_stack.len()-1).unwrap()  as *const u8 as usize;   // Выравниваем адрес. Надо бы использовать нормальную стандартную функцию.

            while(stk_addr % 4) != 0 {stk_addr-=1;}
            stk_addr-=64;


            RTOS_KERNEL.status_pool[RTOS_KERNEL.active_thread_pool[0]] = ContextStatus::Idle;   // Выключаем статус "активно" активного потока.
            RTOS_KERNEL.active_thread_pool[0] = next_el;


/// Если следущий контекст будет работать впервые, прыгнем "грубо" через prepare_task_stack_once
/// Послердник в виде функции позволяет нам вернуть ядро в нормальный режим работы.
/// Если же контекст уже работал раньше, переключимся, выгрузив регистры r4-r11 самостоятельно и дав
/// выгрузить все остальное самому процессору

            match RTOS_KERNEL.status_pool[next_el]
            {
                ContextStatus::Empty =>
                    {
                        RTOS_KERNEL.status_pool[next_el] = ContextStatus::Active;
                        asm!
                        (
                        "str r0,    [sp, #+28]",
                        "mov r0, {0}",
                        "str r0,   [sp, #+24]",
                        "mov lr,    0xFFFFFFF9",
                        "bx  lr",
                        in(reg) prepare_task_stack_once,
                        in("r0") 0b0000_0001_0000_0000_0000_0000_0000_0000,
                        );
                        let _state_reset_reg = 0xE000ED0C as *mut usize;
                    },
                ContextStatus::Idle =>
                    {
                        RTOS_KERNEL.status_pool[next_el] = ContextStatus::Active;
                        let mut stk_addr =kernel_stack.get(kernel_stack.len()-1).unwrap()  as *const u8 as usize;
                        while(stk_addr % 4) != 0 {stk_addr-=1;}
                        stk_addr-=64;

                        asm!
                        (
                        "mov sp, {1}",
                        "mov    lr,    0xFFFFFFFD",
                        "msr    psp,   {0}",
                        "ldm    r0,    {{r4-r11}}",
                        "bx     lr",
                        in(reg) RTOS_KERNEL.thread_pool.el_crate[RTOS_KERNEL.active_thread_pool[0]].elem.sp,
                        in(reg) stk_addr,
                        in("r0") RTOS_KERNEL.thread_pool.el_crate[RTOS_KERNEL.active_thread_pool[0]].elem.rons.as_mut_ptr(),
                        );
                    },
                _=>{},
            }
        }
}
