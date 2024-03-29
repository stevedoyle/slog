// Example showing how to parse yaml using golang. This example uses
// mapstructure instead of the yaml.v3 package to provide some flexibility in
// the yaml construction.
package main

import (
	"fmt"
	"log"
	"os"

	"github.com/mitchellh/mapstructure"
	"gopkg.in/yaml.v3"
)

type People struct {
	People []Person `mapstructure:"people"`
}
type Person struct {
	Name     string   `mapstructure:"name"`
	Age      uint8    `mapstructure:"age"`
	Siblings []string `mapstructure:"siblings"`
}

func main() {
	f, err := os.ReadFile("example.yaml")
	if err != nil {
		log.Fatal(err)
	}

	var p People
	var raw interface{}

	if err := yaml.Unmarshal(f, &raw); err != nil {
		log.Fatal(err)
	}

	// Use mapstructure to convert out interface{} to People (var p)
	decoder, _ := mapstructure.NewDecoder(
		(&mapstructure.DecoderConfig{WeaklyTypedInput: true, Result: &p}))
	if err := decoder.Decode(raw); err != nil {
		log.Fatal(err)
	}

	fmt.Printf("%+v\n", p)
}
