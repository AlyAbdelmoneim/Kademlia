pub trait StorageService<V = String> {
    fn store(&mut self, key: String, value: V) -> Result<(), String>;
    fn get(&self, key: String) -> Option<&V>;
    fn remove(&mut self, key:String) -> Result<(), String>;
    fn contains(&self, key: String) -> bool;
}
