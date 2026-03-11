package connections

import (
	"bufio"
	"context"
	"database/sql"
	"fmt"
	"os"
	"os/exec"
	"strings"
	"time"
)

// Manager orchestrates all connection flows.
type Manager struct {
	store *Store
}

// NewManager creates a Manager backed by db. Calls EnsureTable on init.
func NewManager(db *sql.DB) (*Manager, error) {
	store := NewStore(db)
	if err := store.EnsureTable(context.Background()); err != nil {
		return nil, fmt.Errorf("NewManager: EnsureTable: %w", err)
	}
	return &Manager{store: store}, nil
}

// ConnectOptions controls how Connect behaves.
type ConnectOptions struct {
	Method       AuthMethod            // force a specific method (zero = prompt if multiple)
	OAuthTimeout time.Duration
	FieldValues  map[string]string // pre-filled field values for non-interactive use
}

// Connect authenticates to platform and saves the result. Returns saved Connection.
func (m *Manager) Connect(ctx context.Context, platformID string, opts ConnectOptions) (*Connection, error) {
	// 1. Look up platformID in Registry
	p, ok := Get(platformID)
	if !ok {
		return nil, fmt.Errorf("connect: unknown platform %q", platformID)
	}

	// 2. Pick method
	method := opts.Method
	if method == "" {
		method = m.pickMethod(p)
	}

	// 3. Validate chosen method is in p.Methods
	supported := false
	for _, m := range p.Methods {
		if m == method {
			supported = true
			break
		}
	}
	if !supported {
		return nil, fmt.Errorf("connect: method %q not supported by platform %q", method, platformID)
	}

	conn := &Connection{
		Platform: platformID,
		Method:   method,
		Data:     map[string]interface{}{},
	}

	// 4. Switch on method
	switch method {
	case MethodOAuth:
		if err := m.connectOAuth(ctx, p, conn, opts.OAuthTimeout); err != nil {
			return nil, fmt.Errorf("connect: oauth: %w", err)
		}
	case MethodAPIKey, MethodAppPass, MethodConnStr:
		if err := m.connectFields(p, method, conn, opts.FieldValues); err != nil {
			return nil, fmt.Errorf("connect: fields: %w", err)
		}
	case MethodBrowser:
		return nil, fmt.Errorf("connect: platform %q requires browser-based login — run `monoes connect %s`", platformID, platformID)
	default:
		return nil, fmt.Errorf("connect: unsupported method %q", method)
	}

	// 5. Call ValidateConnection to get accountID
	accountID, err := ValidateConnection(ctx, conn)
	if err != nil {
		return nil, fmt.Errorf("connect: validate: %w", err)
	}

	// 6. Set conn.AccountID, conn.Status="active", conn.LastTested=now
	conn.AccountID = accountID
	conn.Status = "active"
	conn.LastTested = time.Now().UTC().Format(time.RFC3339)

	// 7. Auto-generate conn.Label
	if accountID != "" {
		conn.Label = fmt.Sprintf("%s – %s", p.Name, accountID)
	} else {
		conn.Label = p.Name
	}

	// 8. Save the connection
	if err := m.store.Save(ctx, conn); err != nil {
		return nil, fmt.Errorf("connect: save: %w", err)
	}

	// 9. Print success messages
	fmt.Printf("✓ Connected as %s\n", accountID)
	fmt.Printf("✓ Saved as %s (id: %s)\n", conn.Label, conn.ID)

	// 10. Return conn
	return conn, nil
}

// List returns all connections, optionally filtered by platform.
func (m *Manager) List(ctx context.Context, platform string) ([]Connection, error) {
	if platform == "" {
		return m.store.ListAll(ctx)
	}
	result, err := m.store.ListByPlatform(ctx, platform)
	if err != nil {
		return nil, err
	}
	if result == nil {
		return []Connection{}, nil
	}
	return result, nil
}

// Test re-validates a connection and updates its status.
func (m *Manager) Test(ctx context.Context, id string) error {
	conn, err := m.store.Get(ctx, id)
	if err != nil {
		return fmt.Errorf("test: get connection: %w", err)
	}
	if conn == nil {
		return fmt.Errorf("test: connection %q not found", id)
	}

	accountID, err := ValidateConnection(ctx, conn)
	if err != nil {
		_ = m.store.MarkTested(ctx, id, "error")
		fmt.Printf("✗ Validation failed: %v\n", err)
		return err
	}

	if accountID != "" {
		conn.AccountID = accountID
		p, ok := Get(conn.Platform)
		if ok {
			conn.Label = fmt.Sprintf("%s – %s", p.Name, accountID)
		}
		_ = m.store.Save(ctx, conn)
	}

	if err := m.store.MarkTested(ctx, id, "active"); err != nil {
		return fmt.Errorf("test: mark tested: %w", err)
	}
	fmt.Printf("✓ Connection valid as %s\n", accountID)
	return nil
}

// Remove deletes a connection by ID.
func (m *Manager) Remove(ctx context.Context, id string) error {
	return m.store.Delete(ctx, id)
}

// Refresh re-runs the OAuth flow for a connection and updates stored data.
func (m *Manager) Refresh(ctx context.Context, id string, timeout time.Duration) error {
	conn, err := m.store.Get(ctx, id)
	if err != nil {
		return fmt.Errorf("refresh: get connection: %w", err)
	}
	if conn == nil {
		return fmt.Errorf("refresh: connection %q not found", id)
	}

	if conn.Method != MethodOAuth {
		return fmt.Errorf("refresh: only OAuth connections can be refreshed, got method %q", conn.Method)
	}

	p, ok := Get(conn.Platform)
	if !ok {
		return fmt.Errorf("refresh: unknown platform %q", conn.Platform)
	}

	if err := m.connectOAuth(ctx, p, conn, timeout); err != nil {
		return fmt.Errorf("refresh: oauth: %w", err)
	}

	accountID, err := ValidateConnection(ctx, conn)
	if err != nil {
		return fmt.Errorf("refresh: validate: %w", err)
	}

	conn.AccountID = accountID
	conn.Status = "active"
	conn.LastTested = time.Now().UTC().Format(time.RFC3339)
	if accountID != "" {
		conn.Label = fmt.Sprintf("%s – %s", p.Name, accountID)
	}

	if err := m.store.Save(ctx, conn); err != nil {
		return fmt.Errorf("refresh: save: %w", err)
	}

	return nil
}

// pickMethod prompts the user to select an auth method if there are multiple.
// If only 1 method, returns it directly (no prompt).
func (m *Manager) pickMethod(p PlatformDef) AuthMethod {
	if len(p.Methods) == 1 {
		return p.Methods[0]
	}

	fmt.Printf("Select authentication method for %s:\n", p.Name)
	for i, method := range p.Methods {
		if i == 0 {
			fmt.Printf("  %d) %s (recommended)\n", i+1, method)
		} else {
			fmt.Printf("  %d) %s\n", i+1, method)
		}
	}
	fmt.Print("Choice [1]: ")

	reader := bufio.NewReader(os.Stdin)
	line, _ := reader.ReadString('\n')
	line = strings.TrimSpace(line)
	if line == "" {
		line = "1"
	}

	// Parse the digit
	var idx int
	fmt.Sscanf(line, "%d", &idx)
	idx-- // convert to 0-based
	if idx < 0 || idx >= len(p.Methods) {
		idx = 0
	}

	return p.Methods[idx]
}

// connectOAuth runs the OAuth flow for a platform and populates conn.Data.
func (m *Manager) connectOAuth(ctx context.Context, p PlatformDef, conn *Connection, timeout time.Duration) error {
	if p.OAuth == nil {
		return fmt.Errorf("connectOAuth: platform %q has no OAuth config", p.ID)
	}

	cfg := *p.OAuth // copy

	// Look up env vars MONOES_{UPPERCASE_PLATFORM}_CLIENT_ID and _CLIENT_SECRET
	envPrefix := "MONOES_" + strings.ToUpper(strings.ReplaceAll(p.ID, "-", "_")) + "_"
	if cfg.ClientID == "" {
		cfg.ClientID = os.Getenv(envPrefix + "CLIENT_ID")
	}
	if cfg.ClientSecret == "" {
		cfg.ClientSecret = os.Getenv(envPrefix + "CLIENT_SECRET")
	}

	if cfg.ClientID == "" {
		return fmt.Errorf("connectOAuth: missing ClientID for platform %q (set %sCLIENT_ID)", p.ID, envPrefix)
	}

	result, err := RunOAuthFlow(ctx, cfg, timeout)
	if err != nil {
		return fmt.Errorf("connectOAuth: %w", err)
	}

	// Populate conn.Data
	conn.Data["access_token"] = result.AccessToken
	conn.Data["refresh_token"] = result.RefreshToken
	conn.Data["token_type"] = result.TokenType
	conn.Data["scope"] = result.Scope

	// expires_at as RFC3339
	if result.ExpiresIn > 0 {
		expiresAt := time.Now().UTC().Add(time.Duration(result.ExpiresIn) * time.Second)
		conn.Data["expires_at"] = expiresAt.Format(time.RFC3339)
	} else {
		conn.Data["expires_at"] = ""
	}

	return nil
}

// connectFields prompts the user to fill in credential fields and populates conn.Data.
func (m *Manager) connectFields(p PlatformDef, method AuthMethod, conn *Connection, prefilled map[string]string) error {
	fields, ok := p.Fields[method]
	if !ok {
		return fmt.Errorf("connectFields: platform %q has no fields for method %q", p.ID, method)
	}

	reader := bufio.NewReader(os.Stdin)

	for _, field := range fields {
		// Check prefilled map first
		if prefilled != nil {
			if val, ok := prefilled[field.Key]; ok {
				if field.Required && val == "" {
					return fmt.Errorf("connectFields: required field %q is empty", field.Key)
				}
				conn.Data[field.Key] = val
				continue
			}
		}

		// Print help if available
		if field.HelpText != "" {
			fmt.Printf("  %s\n", field.HelpText)
		}
		if field.HelpURL != "" {
			fmt.Printf("  See: %s\n", field.HelpURL)
		}

		fmt.Printf("%s: ", field.Label)

		var value string
		if field.Secret {
			value = readSecret(reader)
		} else {
			line, _ := reader.ReadString('\n')
			value = strings.TrimSpace(line)
		}

		if field.Required && value == "" {
			return fmt.Errorf("connectFields: required field %q is empty", field.Key)
		}

		if value != "" {
			conn.Data[field.Key] = value
		}
	}

	return nil
}

// readSecret reads a secret value from the terminal with echo disabled.
func readSecret(r *bufio.Reader) string {
	if err := exec.Command("stty", "-echo").Run(); err == nil {
		defer func() {
			_ = exec.Command("stty", "echo").Run()
			fmt.Println()
		}()
	}
	line, _ := r.ReadString('\n')
	return strings.TrimSpace(line)
}
