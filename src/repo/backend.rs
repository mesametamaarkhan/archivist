use anyhow::Result;

pub enum ObjectType {
    Blob,
    Tree,
    Snapshot,
}

pub trait Backend {
    fn put(&self, obj_type: ObjectType, hash: &str, data: &[u8]) -> Result<()>;
    fn get(&self, obj_type: ObjectType, hash: &str) -> Result<Vec<u8>>;
    fn exists(&self, obj_type: ObjectType, hash: &str) -> Result<bool>;
}