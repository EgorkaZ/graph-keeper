# graph-keeper

### Binary usage

To read `.tgf` file call:
```
graph-keeper <filename>
```

Otherwise, input will be consumed from _stdin_

### Library usage

Everything is re-exported in module `graph_keeper`

Key object to work with is `Graph`. You can associate data with both it's verticies and edges. Data associated with verticies is called _vertex data_, with edges — _edge label_, which are the first and the second type parameters of the `Graph` correspondedly.

To read `.tgf` file use `read_tgf`, which accepts iterator of lines and returns `Graph<String, String>`.

To write `.tgf` use `to_tgf`, which returns `String`, that may later be written to the file. The function requires both verticle data and edge labels to implement `Display`.
