use crate::*;
use is_type::Is;

// Monad
pub trait Monad: Applicative + LinearMonad {
    fn fbind<TIn, TOut, F>(
        x: &<Self as WithTypeArg<TIn>>::Type,
        f: F,
    ) -> <Self as WithTypeArg<TOut>>::Type
    where
        Self: WithTypeArg<TIn> + WithTypeArg<TOut>,
        F: Fn(&TIn) -> <Self as WithTypeArg<TOut>>::Type;

    fn fbind_ignore<TIn, TOut>(
        x: &<Self as WithTypeArg<TIn>>::Type,
        y: &<Self as WithTypeArg<TOut>>::Type,
    ) -> <Self as WithTypeArg<TOut>>::Type
    where
        Self: WithTypeArg<TIn> + WithTypeArg<TOut>,
        <Self as WithTypeArg<TOut>>::Type: Clone,
    {
        <Self as Monad>::fbind::<TIn, TOut, _>(x, |_| y.clone())
    }

    fn fjoin<T>(
        x: &<Self as WithTypeArg<<Self as WithTypeArg<T>>::Type>>::Type,
    ) -> <Self as WithTypeArg<T>>::Type
    where
        Self: WithTypeArg<T> + WithTypeArg<<Self as WithTypeArg<T>>::Type>,
        <Self as WithTypeArg<T>>::Type: Clone,
    {
        <Self as Monad>::fbind::<<Self as WithTypeArg<T>>::Type, T, _>(x, |y| y.clone())
    }
}

// LinearMonad
pub trait LinearMonad: LinearFunctor + Lift {
    fn lbind<TIn, TOut, F>(
        x: <Self as WithTypeArg<TIn>>::Type,
        f: F,
    ) -> <Self as WithTypeArg<TOut>>::Type
    where
        Self: WithTypeArg<TIn> + WithTypeArg<TOut>,
        F: Fn(TIn) -> <Self as WithTypeArg<TOut>>::Type;

    fn lbind_ignore<TIn, TOut>(
        x: <Self as WithTypeArg<TIn>>::Type,
        y: &<Self as WithTypeArg<TOut>>::Type,
    ) -> <Self as WithTypeArg<TOut>>::Type
    where
        Self: WithTypeArg<TIn> + WithTypeArg<TOut>,
        <Self as WithTypeArg<TOut>>::Type: Clone,
    {
        <Self as LinearMonad>::lbind::<TIn, TOut, _>(x, |_| Clone::clone(y))
    }

    fn ljoin<T>(
        x: <Self as WithTypeArg<<Self as WithTypeArg<T>>::Type>>::Type,
    ) -> <Self as WithTypeArg<T>>::Type
    where
        Self: WithTypeArg<T> + WithTypeArg<<Self as WithTypeArg<T>>::Type>,
    {
        <Self as LinearMonad>::lbind::<<Self as WithTypeArg<T>>::Type, T, _>(x, |y| y)
    }
}

// fbind(x, f)
pub fn fbind<TCon, TIn, TOut, TArg, F, TResult>(x: &TArg, f: F) -> TResult
where
    TCon: Monad + WithTypeArg<TIn> + WithTypeArg<TOut> + ?Sized,
    TArg: TypeApp<TCon, TIn> + ?Sized,
    F: Fn(&TIn) -> TResult,
    TResult: TypeApp<TCon, TOut>,
{
    Is::from_val(<TCon as Monad>::fbind::<TIn, TOut, _>(x.into_ref(), |y| {
        f(y).into_val()
    }))
}

// fbind_c(x, f)
pub fn fbind_c<TCon, TIn, TOut, F, U>(x: &<TCon as WithTypeArg<TIn>>::Type, f: F) -> U
where
    TCon: Monad + WithTypeArg<TIn> + WithTypeArg<TOut> + ?Sized,
    F: Fn(&TIn) -> U,
    U: TypeApp<TCon, TOut>,
{
    fbind(Is::into_ref(x), f)
}

// fbind_ignore(x, y)
pub fn fbind_ignore<TCon, TIn, TOut, TIgnoreArg, U>(x: &TIgnoreArg, y: &U) -> U
where
    TCon: Monad + WithTypeArg<TIn> + WithTypeArg<TOut> + ?Sized,
    TIgnoreArg: TypeApp<TCon, TIn> + ?Sized,
    U: TypeApp<TCon, TOut>,
    <TCon as WithTypeArg<TOut>>::Type: Clone,
{
    Is::from_val(<TCon as Monad>::fbind_ignore::<TIn, TOut>(
        x.into_ref(),
        y.into_ref(),
    ))
}

// join(x)

// "TInner: TypeApp<TCon, T>" implies "TInner = <TCon as WithTypeArg<TInner>>::Type",
// indeed this is what `Is::into()` and associated functions do.
//
// But here we've got:
//
// "TInner: TypeApp<TCon, T>"
//
// and we want to convert:
//
// X<TInner>
// to
// X<<TCon as WithTypeArg<TInner>>::Type>"
//
// Given the definition of `TypeApp` this should always be true (I believe it's impossible for
// it not to be true) but I can't work out how  to write this safely, without using `fmap`.
//
// But doing `fmap` on the structure will take time,
// whereas I want this to be a zero time operation.
//
// So we're naughty and use `transmute`. I believe this is still sound,
// tell me loudly if you find a case where it isn't.
fn into_functor_ref<TCon, T, TInner>(
    x: &<TCon as WithTypeArg<TInner>>::Type,
) -> &<TCon as WithTypeArg<<TCon as WithTypeArg<T>>::Type>>::Type
where
    TCon: Functor
        + WithTypeArg<T>
        + WithTypeArg<TInner>
        + WithTypeArg<<TCon as WithTypeArg<T>>::Type>
        + ?Sized,
    TInner: TypeApp<TCon, T>,
{
    unsafe { std::mem::transmute(x) }
}

pub fn fjoin<TCon, T, TInner, TArg>(x: &TArg) -> <TCon as WithTypeArg<T>>::Type
where
    TCon: Monad
        + WithTypeArg<T>
        + WithTypeArg<TInner>
        + WithTypeArg<<TCon as WithTypeArg<T>>::Type>
        + ?Sized,
    TInner: TypeApp<TCon, T>,
    TArg: TypeApp<TCon, TInner> + ?Sized,
    <TCon as WithTypeArg<T>>::Type: Clone,
{
    <TCon as Monad>::fjoin::<T>(into_functor_ref::<TCon, T, TInner>(x.into_ref()))
}

// lbind(x, f)
pub fn lbind<TCon, TIn, TOut, X, F, TResult>(x: X, f: F) -> TResult
where
    TCon: LinearMonad + WithTypeArg<TIn> + WithTypeArg<TOut> + ?Sized,
    X: TypeApp<TCon, TIn>,
    F: Fn(TIn) -> TResult,
    TResult: TypeApp<TCon, TOut>,
{
    Is::from_val(<TCon as LinearMonad>::lbind::<TIn, TOut, _>(
        x.into_val(),
        |y| f(y).into_val(),
    ))
}
/*
// lbind_c(x, f)
pub fn lbind_c<TCon, TIn, TOut, F, X>(x: <TCon as WithTypeArg<TIn>>::Type, f: F) -> <TCon as WithTypeArg<TOut>>::Type
where
    TCon: Monad + WithTypeArg<TIn> + WithTypeArg<TOut> + ?Sized,
    F: Fn(TIn) -> <TCon as WithTypeArg<TOut>>::Type
{
    Is::into_val(lbind(x, f))
}
*/

// lbind_ignore(x, y)
pub fn lbind_ignore<TCon, TIn, TOut, X, TResult>(x: X, y: &TResult) -> TResult
where
    TCon: LinearMonad + WithTypeArg<TIn> + WithTypeArg<TOut> + ?Sized,
    X: TypeApp<TCon, TIn>,
    TResult: TypeApp<TCon, TOut>,
    <TCon as WithTypeArg<TOut>>::Type: Clone,
{
    Is::from_val(<TCon as LinearMonad>::lbind_ignore::<TIn, TOut>(
        x.into_val(),
        y.into_ref(),
    ))
}

// This allows you to make a x.bind(f) call which will work which will call either
// fbind or lbind depending on the arguments
pub trait BindExt<TCon, TIn, TOut, F, TFuncIn, TIsRef, TResult>
where
    TCon: LinearMonad + WithTypeArg<TIn> + WithTypeArg<TOut> + ?Sized,
    Self: TypeAppMaybeRef<TCon, TIn, TIsRef>,
    F: Fn(TFuncIn) -> TResult,
    TResult: TypeApp<TCon, TOut>,
{
    fn bind(self, f: F) -> TResult;
}

impl<TCon, TIn, TOut, F, X, TResult> BindExt<TCon, TIn, TOut, F, TIn, Val, TResult> for X
where
    TCon: LinearMonad + WithTypeArg<TIn> + WithTypeArg<TOut> + ?Sized,
    X: TypeApp<TCon, TIn>,
    F: Fn(TIn) -> TResult,
    TResult: TypeApp<TCon, TOut>,
{
    fn bind(self, f: F) -> TResult {
        lbind(self, f)
    }
}

impl<TCon, TIn, TOut, F, X, TResult> BindExt<TCon, TIn, TOut, F, &TIn, Val, TResult> for X
where
    TCon: LinearMonad + WithTypeArg<TIn> + WithTypeArg<TOut> + ?Sized,
    X: TypeApp<TCon, TIn>,
    F: Fn(&TIn) -> TResult,
    TResult: TypeApp<TCon, TOut>,
{
    fn bind(self, f: F) -> TResult {
        lbind(self, |y: TIn| f(&y))
    }
}

impl<TCon, TIn, TOut, F, X, TResult> BindExt<TCon, TIn, TOut, F, TIn, Ref, TResult> for &X
where
    TCon: Monad + WithTypeArg<TIn> + WithTypeArg<TOut> + ?Sized,
    X: TypeApp<TCon, TIn>,
    F: Fn(TIn) -> TResult,
    TResult: TypeApp<TCon, TOut>,
    TIn: Clone,
{
    fn bind(self, f: F) -> TResult {
        fbind(self, |y: &TIn| f(y.clone()))
    }
}

impl<TCon, TIn, TOut, F, X, TResult> BindExt<TCon, TIn, TOut, F, &TIn, Ref, TResult> for &X
where
    TCon: Monad + WithTypeArg<TIn> + WithTypeArg<TOut> + ?Sized,
    X: TypeApp<TCon, TIn>,
    F: Fn(&TIn) -> TResult,
    TResult: TypeApp<TCon, TOut>,
{
    fn bind(self, f: F) -> TResult {
        fbind(self, f)
    }
}

// bind(x, f)
pub fn bind<TCon, TIn, TOut, F, TFuncIn, TIsRef, TResult, X>(x: X, f: F) -> TResult
where
    TCon: LinearMonad + WithTypeArg<TIn> + WithTypeArg<TOut> + ?Sized,
    F: Fn(TFuncIn) -> TResult,
    X: BindExt<TCon, TIn, TOut, F, TFuncIn, TIsRef, TResult>,
    TResult: TypeApp<TCon, TOut>,
{
    X::bind(x, f)
}
