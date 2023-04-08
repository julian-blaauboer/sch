use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::scheduler::Context;

////////////////////////////////////////////////////////////////////////////////
// Create System Macro                                                        //
////////////////////////////////////////////////////////////////////////////////

#[macro_export]
macro_rules! create_system {
    {
        $vis:vis async fn $name:ident($ctx_i:ident: $ctx_t:ty, $data_i:ident: Arc<$data_t:ty>) $body:block
    } => {
        create_system! {
            after = [];
            $vis async fn $name($ctx_i: $ctx_t, $data_i: Arc<$data_t>) $body
        }
    };

    {
        after = [$($after:ident),*];
        $vis:vis async fn $name:ident($ctx_i:ident: $ctx_t:ty, $data_i:ident: Arc<$data_t:ty>) $body:block
    } => {
        $vis fn $name() -> ::sch::system::System<$data_t> {
            ::sch::system::System {
                name: stringify!($name),
                after: &[$(stringify!($after)),*],
                run: Box::new(|ctx, data| Box::pin({
                    async fn $name($ctx_i: $ctx_t, $data_i: Arc<$data_t>) $body

                    $name(ctx, data)
                })),
            }
        }
    };
}

////////////////////////////////////////////////////////////////////////////////
// System                                                                     //
////////////////////////////////////////////////////////////////////////////////

pub type SystemFn<D> =
    Box<dyn FnMut(Context, Arc<D>) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>>;

pub struct System<D> {
    pub name: &'static str,
    pub after: &'static [&'static str],
    pub run: SystemFn<D>,
}
