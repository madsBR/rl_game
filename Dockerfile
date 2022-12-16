FROM scratch
COPY ./target/wasm32-wasi/release/eframe_template.wasm /eframe_template.wasm
ENTRYPOINT [ "eframe_template.wasm" ]