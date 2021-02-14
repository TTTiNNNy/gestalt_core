




#[path = "./desc.rs"]
mod desc;

pub enum ActiveElSetting
{
	NEXT,
	PREV,
	ARBITRARY(usize)
}

pub(crate)fn def_fn(){}

/// Структура для хранения указателей на функции.
/// Такая стуктура может в рантайме удалять/добавлять/исполнять хранимые внутри функции.
/// Таким образом можно создавать функции, которые бубут исполнятся лишь при определенных моментах, а позже будут удалены.
/// Исполнение функций внутри реализовано без ветвлений
#[derive(Copy, Clone)]
pub struct Executor
{
	pub(crate) task_pool: List<fn()>,
	pub(crate) cur:	usize
}

/// Т.к. ембеддед, с хипом у нас некоторые проблемы.
/// Эта структура - двусвязный список.
/// Позже в этом списке можно хранить указатели на функцию, добавлять и удалять их прямо в рантайме.
#[derive(Copy, Clone)]
pub struct List <T:Eq + Copy + Clone>
{
	pub empty:		T,
    pub el_crate:	[Node<T>; desc::TASK_NUM],

}

///Непосредственно, ячейка произвольного типа, входящая в двусвязный список.
#[derive(Debug, Clone,Copy, Eq)]
pub struct Node<T:Eq>
{
    pub elem:	T,
    pub next:	usize,
	pub prev:	usize,
}

///Методы работы по списокм.
pub trait ExecList<T>
{	fn is_el_empty(&mut self, numb: usize) -> bool;
	fn push	(&mut self, obj: T) -> usize;
	fn pop	(&mut self, id: usize);
	fn fetch(&self, id: usize) -> &T ;
}

///Методы для выполнения функций, взодящих в список или сущностей на основе списка.
pub trait Kolotilka
{
	fn exec(&self, token: usize);
	fn set_exec_el(&mut self, mode: ActiveElSetting);
}

///Определяем, как сравнивать между собой Nod'ы - ячейки списка на равенство/неравенство
impl <T:Eq>PartialEq for Node<T>{
	fn eq(&self, other: &Self) -> bool {
		self.elem == other.elem
	}

	fn ne(&self, other: &Self) -> bool {
		self.elem != other.elem
	}
}
///конструктор с базовой инициализацией.
pub fn new() -> Executor
{
	let ar;
	unsafe
	{
	 	ar = [Node {
			elem: def_fn as fn(),
			next: 0,
			prev: 0
		}; desc::TASK_NUM];
	}

	/// логика добавления при пустом и 2< элементах у списка отличается.
	/// В целях уменьшения ветвления и упрощения логики создается список с 2 пустыми связанными значениями,
	/// одно из которых будет представлять собой "вершину",
	/// а добавление от 2-ого элемента и дальше гарантирует нам неизменность положения вершины

	let mut executor =  Executor
	{
		task_pool: List {empty: def_fn as fn(), el_crate: ar},
		cur: 2
	};
	unsafe
	{
		let chain_1 = 0;
		let chain_2 = 1;
		executor.task_pool.el_crate[0].next =chain_2 ;
		executor.task_pool.el_crate[0].prev = chain_2;
		executor.task_pool.el_crate[executor.cur].next = chain_1;
		executor.task_pool.el_crate[executor.cur].prev = chain_1;
	}
	executor
}

unsafe impl  Sync for Node<fn()> {}

/// имплементация методов списка к самому списку.

impl  <T:Eq + Copy>ExecList<T> for List<T>
{
	fn is_el_empty(&mut self, numb: usize) -> bool
	{
		self.el_crate[numb].elem == self.empty

	}

	fn push(&mut self, obj: T) -> usize
	{
		let mut i= 2;
		while !self.is_el_empty(i) {i=i+1;}
		let _new_el = &mut self.el_crate[i];
		let head  = &self.el_crate[0] as *const Node<T> as *mut Node<T>;

		unsafe
			{
				(*&mut self.el_crate[i]).elem = obj;
				(*&mut self.el_crate[i]).next = 0;
				(*&mut self.el_crate[i]).prev = (*head).prev;

				self.el_crate[(*head).prev].next = i;
				(*head).prev = i;
			}
		i
	}

	fn pop(&mut self, id: usize)
	{
		self.el_crate[id].elem= self.empty;
		unsafe
			{
				self.el_crate[self.el_crate[id].prev].next    = self.el_crate[id].next;
				self.el_crate[self.el_crate[id].next].prev    = self.el_crate[id].prev;
			}


	}

	fn fetch(&self, id: usize) -> &T
	{
		&self.el_crate[id].elem

	}
}

/// Реализация методов выполнения задач для соответствующей структуры
impl <'a>Kolotilka for Executor
{
	fn exec(&self, token: usize)
	{
		let el = self.task_pool.el_crate[token];
		unsafe { (el.elem)(); }
	}

	fn set_exec_el(&mut self, mode: ActiveElSetting)
	{
		unsafe
		{
			match mode
			{
				ActiveElSetting::NEXT => {self.cur = self.task_pool.el_crate[self.cur].next;},
				ActiveElSetting::ARBITRARY(elem_numb) => {self.cur =
					elem_numb},
				ActiveElSetting::PREV => {self.cur = self.task_pool.el_crate[self.cur].prev}
			}
		}
	}
}

/// Релизация методов списка для стуктуры-выполнителя. Таким образом их методы прямо совместимы.
impl ExecList<fn()> for Executor{
	fn is_el_empty(&mut self, numb: usize) -> bool {
		self.task_pool.is_el_empty(numb)
	}

	fn push(&mut self, obj: fn()) -> usize {
		self.task_pool.push(obj)
	}

	fn pop(&mut self, id: usize) {
		self.pop(id);
	}

	fn fetch(&self, id: usize) -> &fn() {
		self.fetch(id)
	}
}

/// Имплементация функции выполнения всех имеющихся функций внутри к структуре-исполнителю.
impl Executor
{
	pub fn exec_all(&self)
	{
		//unsafe {((*self.cur).elem)() ;}
		let mut el = self.task_pool.el_crate[0].next;
		while el != 0
		{
			unsafe
				{
					(self.task_pool.el_crate[el].elem)();//((*el).elem)();
					el = self.task_pool.el_crate[el].next;//(*el).next;
				}

		}
	}
}
