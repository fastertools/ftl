package main

import (
	"os"

	"github.com/fastertools/ftl-cli/go/spin-compose/cmd"
)

func main() {
	if err := cmd.Execute(); err != nil {
		os.Exit(1)
	}
}