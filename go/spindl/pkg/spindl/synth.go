package spindl

import (
	"github.com/fastertools/ftl-cli/go/spindl/internal/synth"
)

// Engine wraps the internal synthesis engine
type Engine struct {
	internal *synth.Engine
}

// NewEngine creates a new synthesis engine
func NewEngine() *Engine {
	return &Engine{
		internal: synth.NewEngine(),
	}
}

// SynthesizeConfig synthesizes a configuration file to spin.toml format
func (e *Engine) SynthesizeConfig(configData []byte, format string) ([]byte, error) {
	return e.internal.SynthesizeConfig(configData, format)
}