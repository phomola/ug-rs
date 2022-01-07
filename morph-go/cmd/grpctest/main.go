package main

import (
	"context"
	"fmt"

	"github.com/phomola/ug-rs/morph-go/morphrpc"
	"google.golang.org/grpc"
)

func main() {
	conn, err := grpc.Dial("localhost:8080", grpc.WithInsecure())
	if err != nil {
		panic(err)
	}
	defer conn.Close()
	client := morphrpc.NewServiceClient(conn)
	resp, err := client.Analyse(context.Background(), &morphrpc.AnalyseRequest{Input: "input text"})
	if err != nil {
		panic(err)
	}
	for _, item := range resp.GetItems() {
		fmt.Println("form:", item.GetForm())
		for _, e := range item.GetEntries() {
			fmt.Println("-", e.GetLemma(), e.GetTagSet().GetPos(), e.GetTagSet().GetTags())
		}
	}
}
