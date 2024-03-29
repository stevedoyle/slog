// Implements an abstract dataplane interface using a Bridge pattern example in
// golang.
package main

import (
	"fmt"
	"log"
	"log/slog"
)

// An absract dataplane interface. The interface methods represent the set of
// operations permitted on the dataplane.
type Dataplane interface {
	CreateVrf(name string, vrfid uint16)
	DeleteVrf(vrfid uint16)

	AddVrfInterface(vrfid uint16, netif NetifCfg)
	RemoveVrfInterface(vrfid uint16, ifid uint16)
}

// Network Interface configuration
type NetifCfg struct {
	Name   string
	Id     uint16
	Iftype string
}

// A VPP dataplane
type VppDataplane struct {
}

func (vpp VppDataplane) CreateVrf(name string, vrfid uint16) {
	slog.Info("[VPP] CreateVrf", "name", name, "vrfid", vrfid)
}

func (vpp VppDataplane) DeleteVrf(vrfid uint16) {
	slog.Info("[VPP] DeleteVrf", "vrfid", vrfid)
}

func (vpp VppDataplane) AddVrfInterface(vrfid uint16, netif NetifCfg) {
	slog.Info("[VPP] AddVrfInterface", "vrfid", vrfid, "netif", netif)
}

func (vpp VppDataplane) RemoveVrfInterface(vrfid uint16, ifid uint16) {
	slog.Info("[VPP] RemoveVrfInterface", "vrfid", vrfid, "ifid", ifid)
}

// A Linux dataplane
type LinuxDataplane struct {
}

func (vpp LinuxDataplane) CreateVrf(name string, vrfid uint16) {
	slog.Info("[Linux] CreateVrf", "name", name, "vrfid", vrfid)
}

func (vpp LinuxDataplane) DeleteVrf(vrfid uint16) {
	slog.Info("[Linux] DeleteVrf", "vrfid", vrfid)
}

func (vpp LinuxDataplane) AddVrfInterface(vrfid uint16, netif NetifCfg) {
	slog.Info("[Linux] AddVrfInterface", "vrfid", vrfid, "netif", netif)
}

func (vpp LinuxDataplane) RemoveVrfInterface(vrfid uint16, ifid uint16) {
	slog.Info("[Linux] RemoveVrfInterface", "vrfid", vrfid, "ifid", ifid)
}

// A factory for creating dataplanes
func CreateDataplane(dpType string) (Dataplane, error) {
	switch dpType {
	case "vpp":
		return &VppDataplane{}, nil
	case "linux":
		return &LinuxDataplane{}, nil
	default:
		return nil, fmt.Errorf("unsupported dataplane type")
	}
}

// Example configuration of a dataplane
func useDataplane(dp Dataplane) {
	netif := NetifCfg{
		Name:   "xfrm10",
		Id:     10,
		Iftype: "ipip_tunnel",
	}

	dp.CreateVrf("blue", 1)
	dp.AddVrfInterface(1, netif)
	dp.RemoveVrfInterface(1, 10)
	dp.DeleteVrf(1)
}

func main() {
	slog.SetLogLoggerLevel(slog.LevelInfo)
	dp, err := CreateDataplane("linux")
	if err != nil {
		log.Fatal("Error creating dataplane: ", err)
	}
	useDataplane(dp)

	dp, err = CreateDataplane("linux")
	if err != nil {
		log.Fatal("Error creating dataplane: ", err)
	}
	useDataplane(dp)
}
