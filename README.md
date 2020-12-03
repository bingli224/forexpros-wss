# forexpros-wss
Simple secured websocket (wss) client to forexpros.com stream--a realtime source for investing.com.

Pair id examples:
 	"945629"	BTC/USD
 	"1058142"	ETC/USD
 	"8984"	Hang Seng Futures
 	"8873"	Dow Jones Industrial Average (DJI)
 	"14958"	NASDAQ Composite
 	"8830"	Gold Futures
 
For further pair id, hack the websocket in some browser debugger, such as Chrome inspect.

# Example

```
cargo run --example watch
```

Or

```
use forexpros_wss::push;

fn main ( ) {
	let pair_id = "945629";	// BTC/USD
	
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
