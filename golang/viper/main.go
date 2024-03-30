// Example using viper to manage configuration for a Go application
package main

import (
	"fmt"
	"log"

	"github.com/spf13/viper"
)

type Networks struct {
	NetIfs map[string]NetIf `mapstructure:"interfaces"`
}

type NetIf struct {
	Id   uint16 `mapstructure:"id"`
	Type string `mapstructure:"type"`
}

func main() {
	viper.SetConfigName("config") // name of config file
	viper.SetConfigType("yaml")
	viper.AddConfigPath("./")

	if err := viper.ReadInConfig(); err != nil {
		if _, ok := err.(viper.ConfigFileNotFoundError); ok {
			log.Fatal(err)
		} else {
			// Config file was found but some other error occurred.
			log.Fatal(err)
		}
	}
	fmt.Println("Config file read successfully")

	fmt.Println("verbose: ", viper.GetBool("verbose"))
	fmt.Println("loglevel:", viper.GetString("loglevel"))
	fmt.Println("networks:", viper.GetStringMap("networks"))
	networks := viper.GetStringMap("networks")
	for k, v := range networks {
		fmt.Println(k, "->", v)
		interfaces := viper.GetStringMap("networks." + k + ".interfaces")
		for i, cfg := range interfaces {
			fmt.Println("  ", i, "->", cfg)
		}
	}

	nets := make(map[string]Networks)
	if err := viper.UnmarshalKey("networks", &nets); err != nil {
		log.Fatal(err)
	}

	fmt.Println(nets)
	fmt.Println("Interface ID:", nets["blue"].NetIfs["xfrm10"].Id)

	fmt.Print("Networks: ")
	for netname := range nets {
		fmt.Print(netname, " ")
	}
	fmt.Println()

	fmt.Println("Interfaces:")
	for netname, net := range nets {
		for ifname := range net.NetIfs {
			fmt.Printf("  %s.%s\n", netname, ifname)
		}
	}

	// viper.OnConfigChange(func(e fsnotify.Event) {
	// 	fmt.Println("Config file changed:", e.Name)
	// })
	// viper.WatchConfig()

	// wait forever
	// <-make(chan struct{})
}
