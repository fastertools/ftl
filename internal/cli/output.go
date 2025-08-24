package cli

import (
	"encoding/json"
	"fmt"
	"io"
	"text/tabwriter"

	"github.com/fastertools/ftl/internal/constants"
)

// OutputFormat represents the output format type
type OutputFormat string

const (
	OutputFormatTable OutputFormat = "table"
	OutputFormatJSON  OutputFormat = "json"
)

// DataWriter handles formatted output of structured data
type DataWriter struct {
	output io.Writer
	format OutputFormat
}

// NewDataWriter creates a new DataWriter
func NewDataWriter(output io.Writer, format string) *DataWriter {
	of := OutputFormatTable
	if format == "json" {
		of = OutputFormatJSON
	}
	return &DataWriter{
		output: output,
		format: of,
	}
}

// WriteKeyValue writes key-value pairs in the specified format
func (dw *DataWriter) WriteKeyValue(title string, data map[string]interface{}) error {
	switch dw.format {
	case OutputFormatJSON:
		return dw.writeJSON(data)
	case OutputFormatTable:
		return dw.writeKeyValueTable(title, data)
	default:
		return fmt.Errorf("unsupported output format: %s", dw.format)
	}
}

// WriteTable writes tabular data with headers
func (dw *DataWriter) WriteTable(headers []string, rows [][]string) error {
	switch dw.format {
	case OutputFormatJSON:
		// Convert to array of objects for JSON
		var jsonData []map[string]string
		for _, row := range rows {
			obj := make(map[string]string)
			for i, header := range headers {
				if i < len(row) {
					obj[header] = row[i]
				}
			}
			jsonData = append(jsonData, obj)
		}
		return dw.writeJSON(jsonData)
	case OutputFormatTable:
		return dw.writeTabularData(headers, rows)
	default:
		return fmt.Errorf("unsupported output format: %s", dw.format)
	}
}

// WriteStruct writes a struct in the specified format
func (dw *DataWriter) WriteStruct(data interface{}) error {
	switch dw.format {
	case OutputFormatJSON:
		return dw.writeJSON(data)
	case OutputFormatTable:
		// For table format, we need to convert struct to key-value pairs
		// This is a simplified version - could be enhanced with reflection
		return fmt.Errorf("table format not supported for arbitrary structs - use WriteKeyValue or WriteTable")
	default:
		return fmt.Errorf("unsupported output format: %s", dw.format)
	}
}

// writeJSON writes data as JSON
func (dw *DataWriter) writeJSON(data interface{}) error {
	encoder := json.NewEncoder(dw.output)
	encoder.SetIndent("", "  ")
	return encoder.Encode(data)
}

// writeKeyValueTable writes key-value pairs as an aligned table
func (dw *DataWriter) writeKeyValueTable(title string, data map[string]interface{}) error {
	if title != "" {
		_, _ = fmt.Fprintln(dw.output)
		_, _ = fmt.Fprintln(dw.output, title)
	}

	// Use tabwriter for consistent alignment
	w := tabwriter.NewWriter(dw.output, 0, 0, 2, ' ', 0)

	// Create ordered list of keys for consistent output
	orderedKeys := constants.OrderedKeys

	// Print known keys in order first
	for _, key := range orderedKeys {
		if value, exists := data[key]; exists && value != nil && value != "" {
			_, _ = fmt.Fprintf(w, "  %s:\t%v\t\n", key, value)
		}
	}

	// Print any remaining keys
	for key, value := range data {
		found := false
		for _, orderedKey := range orderedKeys {
			if key == orderedKey {
				found = true
				break
			}
		}
		if !found && value != nil && value != "" {
			_, _ = fmt.Fprintf(w, "  %s:\t%v\t\n", key, value)
		}
	}

	_ = w.Flush()
	_, _ = fmt.Fprintln(dw.output)
	return nil
}

// writeTabularData writes headers and rows as a table
func (dw *DataWriter) writeTabularData(headers []string, rows [][]string) error {
	_, _ = fmt.Fprintln(dw.output)

	// Use tabwriter for consistent alignment
	w := tabwriter.NewWriter(dw.output, 0, 0, 2, ' ', 0)

	// Write headers
	for i, header := range headers {
		_, _ = fmt.Fprint(w, header)
		if i < len(headers)-1 {
			_, _ = fmt.Fprint(w, "\t")
		}
	}
	_, _ = fmt.Fprintln(w, "\t") // Trailing tab for proper termination

	// Write rows
	for _, row := range rows {
		for i, cell := range row {
			_, _ = fmt.Fprint(w, cell)
			if i < len(row)-1 {
				_, _ = fmt.Fprint(w, "\t")
			}
		}
		_, _ = fmt.Fprintln(w, "\t") // Trailing tab for proper termination
	}

	_ = w.Flush()
	_, _ = fmt.Fprintln(dw.output)
	return nil
}

// TableBuilder helps build table data incrementally
type TableBuilder struct {
	headers []string
	rows    [][]string
}

// NewTableBuilder creates a new TableBuilder
func NewTableBuilder(headers ...string) *TableBuilder {
	return &TableBuilder{
		headers: headers,
		rows:    [][]string{},
	}
}

// AddRow adds a row to the table
func (tb *TableBuilder) AddRow(values ...string) *TableBuilder {
	tb.rows = append(tb.rows, values)
	return tb
}

// Write outputs the table using the DataWriter
func (tb *TableBuilder) Write(dw *DataWriter) error {
	return dw.WriteTable(tb.headers, tb.rows)
}

// KeyValueBuilder helps build key-value data
type KeyValueBuilder struct {
	title string
	data  map[string]interface{}
}

// NewKeyValueBuilder creates a new KeyValueBuilder
func NewKeyValueBuilder(title string) *KeyValueBuilder {
	return &KeyValueBuilder{
		title: title,
		data:  make(map[string]interface{}),
	}
}

// Add adds a key-value pair
func (kvb *KeyValueBuilder) Add(key string, value interface{}) *KeyValueBuilder {
	kvb.data[key] = value
	return kvb
}

// AddIf conditionally adds a key-value pair
func (kvb *KeyValueBuilder) AddIf(condition bool, key string, value interface{}) *KeyValueBuilder {
	if condition {
		kvb.data[key] = value
	}
	return kvb
}

// Write outputs the key-value data using the DataWriter
func (kvb *KeyValueBuilder) Write(dw *DataWriter) error {
	return dw.WriteKeyValue(kvb.title, kvb.data)
}
