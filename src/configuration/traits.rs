use anyhow::Result;

pub trait PostLoadable {
    fn after_load_init(&mut self) -> Result<()>;
}
