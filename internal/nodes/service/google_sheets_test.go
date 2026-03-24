package service

import (
	"testing"
)

func TestSheetsValuesToItems_RowIndex(t *testing.T) {
	values := []interface{}{
		[]interface{}{"title", "status"},
		[]interface{}{"Post 1", "todo"},
		[]interface{}{"Post 2", "done"},
	}

	items := sheetsValuesToItems(values, true)

	if len(items) != 2 {
		t.Fatalf("expected 2 items, got %d", len(items))
	}

	if got := items[0].JSON["_row_index"]; got != 2 {
		t.Errorf("items[0]._row_index = %v, want 2", got)
	}
	if got := items[1].JSON["_row_index"]; got != 3 {
		t.Errorf("items[1]._row_index = %v, want 3", got)
	}

	// Without header row: first data row = sheet row 1
	itemsNoHeader := sheetsValuesToItems(values, false)
	if got := itemsNoHeader[0].JSON["_row_index"]; got != 1 {
		t.Errorf("no-header items[0]._row_index = %v, want 1", got)
	}
}
