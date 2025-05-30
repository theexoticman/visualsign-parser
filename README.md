# visualsign-parser

This repo contains an enclave application to parse unsigned transactions and return VisualSign output.

## Running tests

```
make -C src test
```

## Running locally

The following will start 3 sub-processes: an enclave simulator, a host, and an inner app:

```
make -C src parser
```

The parser will expose a gRPC interface on port `44020` by default.

Once it's up and running, make requests with:

```
grpcurl -plaintext -d '{"unsigned_payload": "0xabcdef"}'  localhost:44020 parser.ParserService/Parse
```

If everything works you'll get a response like this:

```
{
  "parsedTransaction": {
    "payload": {
      "transactionMetadata": [
        ...
      ]
    }
  }
}
```
