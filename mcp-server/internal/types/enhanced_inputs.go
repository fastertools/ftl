package types

// Enhanced input types that embed CLI options directly to avoid import cycles

// UpOptionsEmbedded contains the CLI options for up command (embedded to avoid import cycle)
type UpOptionsEmbedded struct {
	Build                 bool     `json:"build,omitempty"`
	Watch                 bool     `json:"watch,omitempty"`
	SkipSynth            bool     `json:"skip_synth,omitempty"`
	ConfigFile           string   `json:"config_file,omitempty"`
	ComponentIds         []string `json:"component_ids,omitempty"`
	CacheDir             string   `json:"cache_dir,omitempty"`
	DirectMounts         bool     `json:"direct_mounts,omitempty"`
	Env                  []string `json:"env,omitempty"`
	From                 string   `json:"from,omitempty"`
	Insecure             bool     `json:"insecure,omitempty"`
	Temp                 string   `json:"temp,omitempty"`
	AllowTransientWrite  bool     `json:"allow_transient_write,omitempty"`
	Cache                string   `json:"cache,omitempty"`
	DisableCache         bool     `json:"disable_cache,omitempty"`
	DisablePooling       bool     `json:"disable_pooling,omitempty"`
	Follow               []string `json:"follow,omitempty"`
	KeyValue             []string `json:"key_value,omitempty"`
	LogDir               string   `json:"log_dir,omitempty"`
	MaxInstanceMemory    string   `json:"max_instance_memory,omitempty"`
	Quiet                bool     `json:"quiet,omitempty"`
	RuntimeConfigFile    string   `json:"runtime_config_file,omitempty"`
	Sqlite               []string `json:"sqlite,omitempty"`
	StateDir             string   `json:"state_dir,omitempty"`
	Listen               string   `json:"listen,omitempty"`
}

// ToArgs converts UpOptionsEmbedded to command line arguments
func (opts *UpOptionsEmbedded) ToArgs() []string {
	var args []string
	
	if opts.Build {
		args = append(args, "--build")
	}
	if opts.Watch {
		args = append(args, "--watch")
	}
	if opts.SkipSynth {
		args = append(args, "--skip-synth")
	}
	if opts.ConfigFile != "" {
		args = append(args, "--config", opts.ConfigFile)
	}
	for _, id := range opts.ComponentIds {
		args = append(args, "--component-id", id)
	}
	if opts.CacheDir != "" {
		args = append(args, "--cache-dir", opts.CacheDir)
	}
	if opts.DirectMounts {
		args = append(args, "--direct-mounts")
	}
	for _, e := range opts.Env {
		args = append(args, "--env", e)
	}
	if opts.From != "" {
		args = append(args, "--from", opts.From)
	}
	if opts.Insecure {
		args = append(args, "--insecure")
	}
	if opts.Temp != "" {
		args = append(args, "--temp", opts.Temp)
	}
	if opts.AllowTransientWrite {
		args = append(args, "--allow-transient-write")
	}
	if opts.Cache != "" {
		args = append(args, "--cache", opts.Cache)
	}
	if opts.DisableCache {
		args = append(args, "--disable-cache")
	}
	if opts.DisablePooling {
		args = append(args, "--disable-pooling")
	}
	for _, f := range opts.Follow {
		args = append(args, "--follow", f)
	}
	for _, kv := range opts.KeyValue {
		args = append(args, "--key-value", kv)
	}
	if opts.LogDir != "" {
		args = append(args, "--log-dir", opts.LogDir)
	}
	if opts.MaxInstanceMemory != "" {
		args = append(args, "--max-instance-memory", opts.MaxInstanceMemory)
	}
	if opts.Quiet {
		args = append(args, "--quiet")
	}
	if opts.RuntimeConfigFile != "" {
		args = append(args, "--runtime-config", opts.RuntimeConfigFile)
	}
	for _, s := range opts.Sqlite {
		args = append(args, "--sqlite", s)
	}
	if opts.StateDir != "" {
		args = append(args, "--state-dir", opts.StateDir)
	}
	if opts.Listen != "" {
		args = append(args, "--listen", opts.Listen)
	}
	
	return args
}

// BuildOptionsEmbedded contains the CLI options for build command (embedded to avoid import cycle)
type BuildOptionsEmbedded struct {
	SkipSynth  bool   `json:"skip_synth,omitempty"`
	ConfigFile string `json:"config_file,omitempty"`
}

// ToArgs converts BuildOptionsEmbedded to command line arguments
func (opts *BuildOptionsEmbedded) ToArgs() []string {
	var args []string
	
	if opts.SkipSynth {
		args = append(args, "--skip-synth")
	}
	if opts.ConfigFile != "" {
		args = append(args, "--config", opts.ConfigFile)
	}
	
	return args
}