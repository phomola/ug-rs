package main

import (
	"context"
	"net"
	"os"
	"strings"

	"github.com/phomola/textkit"
	"github.com/phomola/ug-rs/morph-go"
	"github.com/phomola/ug-rs/morph-go/morphrpc"
	"google.golang.org/grpc"
)

var tokeniser = new(textkit.Tokeniser)

type serviceServer struct {
	morphrpc.UnimplementedServiceServer
}

func (src *serviceServer) Analyse(ctx context.Context, req *morphrpc.AnalyseRequest) (*morphrpc.AnalyseReply, error) {
	tokens := tokeniser.Tokenise(req.GetInput())
	items := make([]*morphrpc.Item, 0, len(tokens))
	for _, token := range tokens {
		if token.Type != textkit.EOF {
			form := string(token.Form)
			entries, err := morph.Analyse(strings.ToLower(form))
			item := &morphrpc.Item{Form: form}
			if err == nil {
				item.Entries = make([]*morphrpc.Entry, len(entries))
				for i, e := range entries {
					item.Entries[i] = e.ToProto()
				}
			} else {
				item.Error = err.Error()
			}
			items = append(items, item)
		}
	}
	return &morphrpc.AnalyseReply{Items: items}, nil
}

func main() {
	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}
	l, err := net.Listen("tcp", ":"+port)
	if err != nil {
		panic(err)
	}
	server := grpc.NewServer()
	morphrpc.RegisterServiceServer(server, new(serviceServer))
	server.Serve(l)
}
