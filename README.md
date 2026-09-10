## An Async Linkchecker for Mardown Files.

### Overview
- This program aims to read a markdown file from the user, parse the file to find out links that are in the 
markdown format and make a request to those urls. If the URL is still alive and responding then the HTML
response is parsed and the title HTML tag is taken out. The HTML title is along with the URL is pasted in the
an output file otherwise and error response is written to the output file.

### Inner Workings
- Runs 32 thread Tokio thread asynchronously at all times, processing 32 different URLs at any given time.
As soon as one thread returns either successfully or with error another Tokio thread is spawned with the URLs
backed up in the queue.

### Steps to run

1. Build using this command -> `make build`
2. Run the generated binary using -> `./target/debug/rust_async_linkchecker`
3. If you want pass your own input and output files then run this command -> `./target/debug/rust_async_linkchecker </path/to/input_file.md> </path/to/outputfile.md>`

### Alternate way to run 
1. Build using this command -> `cargo run <path/to/inputfile.md> <path/to/outputfile.md>`