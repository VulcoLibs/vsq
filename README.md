# VSQ
Library to discover and query [Rust game](https://rust.facepunch.com/) servers.

# Examples
Querying a know Rust server to receive the info:
```rust
fn main() {
    let sq = ServerQuery::new(SocketAddr::new(
        IpAddr::from([192, 168, 18, 8]),
        28016,
    )).await.unwrap();

    let res = sq.a2s_info().await.unwrap();
}
```

Querying the Master Server for server list:
```rust
fn main() {
    let mq = MasterQuery::new().await;

    let (tx, mut rx) = tokio::sync::mpsc::channel(12);

    let vsq_task = mq.start("hl2master.steampowered.com", 27011, Filters {
        app_id: 252490,
        no_password: true,
    }, tx.clone()).await.unwrap();

    while let Some(addr) = rx.recv().await {
        println!("{}", addr);
    }
}
```
