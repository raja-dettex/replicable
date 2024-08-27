use serde::Serialize;
use std::fmt::Debug;

#[derive(Debug)]
pub struct SerializerError {}
pub struct Row<T> 
where T : Serialize + for <'a> serde::de::Deserialize<'a> +  Debug
{ 
    row: T
}

impl<T> Row<T> 
where T : Serialize + for <'a> serde::de::Deserialize<'a> + Debug { 
    pub fn serialize(&self) -> Result<String, SerializerError> { 
        let Ok(result) = serde_json::to_string(&self.row) else {
            return Err(SerializerError {  })
        };
        Ok(result)
    }
    pub fn deserialize(str : &str) -> Result<T, SerializerError> {
        let Ok(result) = serde_json::from_str(str) else {
            return Err(SerializerError {  })
        };
        Ok(result)
    }
}


// sink writer is generic over a type which implements Writer trait to write to a sink
pub struct SinkWriter<T: Writer> { 
    pub t : T
}


impl<T:Writer> SinkWriter<T> {
    pub async fn write(&self, file: tokio::fs::File) -> Result<(), WriteError> { 
        let a = self.t.write(file).await;
        a
    }
}
#[derive(Debug)] 
pub struct WriteError {
     
}


pub trait Writer { 
    fn write(&self, file: tokio::fs::File) -> impl std::future::Future<Output = Result<(),WriteError>> + std::marker::Send;
}
