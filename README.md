# forexpros-wss
Simple secured websocket (wss) client to forexpros.com stream--a realtime source for investing.com.

# Example

```
use forexpros::push;

fn main ( ) {
	let pair_id = "945629";	// BTC/USD
	//let pair_id = "8984";	// HK50 future
	
	let handler = |s| {
		println ! ( "INFO: {:?}", s );
	};

	let stream = push::Stream::new ( pair_id.to_string ( ), handler )
		.expect ( "Failed to create stream" );
	
	println ! ("main: stream.pair_id={}", stream.pair_id);
	println ! ("main: stream.handler={:?}", stream.stream_handle_spawn);
	
	tokio::runtime::Runtime::new ( )
		.unwrap ( )
		.block_on (
			stream.stream_handle_spawn
		).unwrap ( ).unwrap ( );
}
```
