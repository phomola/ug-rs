# ug-rs

## Morphology

For English morphology there's a REST API server written in Go.

Example request: `{"input": "input text"}`

Example response:
```
{"items":[
  {"form":"input","entries":[
    {"lemma":"input","tagset":{"pos":"ADJECTIVE"}},
    {"lemma":"input","tagset":{"pos":"NOUN","tags":["narr","sg"]}},
    {"lemma":"input","tagset":{"pos":"VERB","tags":["pp"]}}]},
  {"form":"text","entries":[
    {"lemma":"text","tagset":{"pos":"NOUN","tags":["narr","sg"]}},
    {"lemma":"text","tagset":{"pos":"VERB","tags":["inf"]}}]}]}
```

## Parser

The parser is a **unification grammar** parser written in Rust.
It is designed for parsing polysynthetic languages ([such as Aymara](https://aclanthology.org/W13-3712.pdf))
and languages with detached long-distance auxiliaries (such as we find in Eastern Armenian, [Udi](https://www.amazon.com/Endoclitics-Origins-Morphosyntax-Alice-Harris/dp/0199246335) or some Iranian languages such as Talysh).

The parser uses a **chart** and the **Knuth-Bendix completion** procedure for constraint resolution via a confluent rewriting system.

The output of the parser is similar to that of **Slot grammar** ([Deep parsing in IBM Watson](https://dl.acm.org/doi/10.1147/JRD.2012.2185409)) and is meant to be used by an abductive theorem prover ([Hobbs' interpretation as abduction](https://aclanthology.org/C12-1079/)).
