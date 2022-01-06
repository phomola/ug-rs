package main

import (
	"fmt"

	"github.com/phomola/ug-rs/morph-go"
)

func main() {
	m, err := morph.Analyse("books")
	if err != nil {
		panic(err)
	}
	fmt.Println(m)
}
