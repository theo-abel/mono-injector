pub(crate) mod api;
pub(crate) mod module;

pub(crate) use api::MonoSession;

use crate::error::{Error, Result};
use crate::injector::{AssemblyHandle, EjectRequest, InjectRequest};

/// Executes the 7-step inject sequence.
pub(crate) fn inject_steps(
    mut session: MonoSession,
    req: &InjectRequest<'_>,
) -> Result<AssemblyHandle> {
    session.attach()?;
    let image = session.open_image(req.assembly)?;
    let assembly = session.open_assembly(image)?;
    let image = session.get_image(assembly)?;
    let class = session.get_class(image, req.namespace, req.class_name)?;
    let method = session.get_method(class, req.method_name)?;

    session.invoke(method)?;

    AssemblyHandle::new(assembly).ok_or(Error::AssemblyLoadFailed)
}

/// Executes the 5-step eject sequence.
pub(crate) fn eject_steps(mut session: MonoSession, req: &EjectRequest<'_>) -> Result<()> {
    session.attach()?;
    let image = session.get_image(req.handle.as_ptr())?;
    let class = session.get_class(image, req.namespace, req.class_name)?;
    let method = session.get_method(class, req.method_name)?;

    session.invoke(method)?;
    session.close_assembly(req.handle.as_ptr())
}
