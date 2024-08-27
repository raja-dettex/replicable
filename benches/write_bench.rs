use criterion::{Criterion, criterion_group, criterion_main};
use futures::io::Write;
use tokio::runtime::Runtime; 
use tokio::io::AsyncWriteExt;
use replicable::sink::sink::{SinkWriter, WriteError, Writer};
use tokio::fs::OpenOptions;
use serde::{Serialize , Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct TestMessage { 
    message_id: i32,
    message: String
}

impl Writer for TestMessage {
    async fn write(&self, mut file: tokio::fs::File) -> Result<(), WriteError> {
        let b = serde_json::to_vec(&self).unwrap();
        if let Err(err) = file.write_all(&b).await { 
            return Err(WriteError{});
        };
        if let Err(err) = file.write_all(b"\n").await { 
            return Err(WriteError{});
        };
        if let Err(err) = file.flush().await { 
            return Err(WriteError{});
        };
        Ok(())
        
    }
}


fn bench_write(c : &mut Criterion) { 
    let rt = Runtime::new().unwrap();
    c.bench_function("file_write", move|b| { 
        b.iter(|| {
            rt.block_on(async move {
                let m = TestMessage { message_id : 1, message: "this is a test message".to_string()};
                let file = OpenOptions::new().append(true).write(true).create(true).open("./test_output.bin").await.unwrap();
                let file_sink = SinkWriter::<TestMessage>{t: m};
                match file_sink.write(file).await { 
                    Ok(_) => (),
                    Err(err) => eprintln!("file write error : {err:?}")
                }               
            })
        })
    });
}

criterion_group!(benches, bench_write);
criterion_main!(benches);