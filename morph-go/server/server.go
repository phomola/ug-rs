package main

import (
	"encoding/json"
	"log"
	"net/http"
	"strings"

	"github.com/phomola/textkit"
	"github.com/phomola/ug-rs/morph-go"
)

type MorphRequest struct {
	Input string `json:"input"`
}

type Item struct {
	Form    string         `json:"form"`
	Entries []*morph.Entry `json:"entries,omitempty"`
	Error   string         `json:"error,omitempty"`
}

type MorphResponse struct {
	Items []*Item `json:"items"`
}

var tokeniser = new(textkit.Tokeniser)

type MorphologyServer struct{}

func (srv *MorphologyServer) ServeHTTP(w http.ResponseWriter, req *http.Request) {
	in := new(MorphRequest)
	if err := json.NewDecoder(req.Body).Decode(in); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	tokens := tokeniser.Tokenise(in.Input)
	items := make([]*Item, 0, len(tokens))
	for _, token := range tokens {
		if token.Type != textkit.EOF {
			form := string(token.Form)
			entries, err := morph.Analyse(strings.ToLower(form))
			item := &Item{Form: form}
			if err == nil {
				item.Entries = entries
			} else {
				item.Error = err.Error()
			}
			items = append(items, item)
		}
	}
	if err := json.NewEncoder(w).Encode(&MorphResponse{Items: items}); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
}

func main() {
	mux := http.NewServeMux()
	mux.Handle("/morph", new(MorphologyServer))
	if err := http.ListenAndServe(":8080", mux); err != nil && err != http.ErrServerClosed {
		log.Fatal(err)
	}
}
