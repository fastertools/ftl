package auth

import (
	"fmt"
	"strings"
	"time"

	"github.com/golang-jwt/jwt/v5"
)

// JWTClaims represents the claims we extract from the JWT
type JWTClaims struct {
	// Standard claims
	Subject   string `json:"sub"`
	Email     string `json:"email"`
	Name      string `json:"name"`
	ExpiresAt int64  `json:"exp"`
	IssuedAt  int64  `json:"iat"`
	
	// WorkOS-specific claims
	OrganizationID string   `json:"org_id"`
	Organizations  []string `json:"org_ids"`
	ActorType      string   `json:"actor_type"`
	UserID         string   `json:"user_id"`
	
	// Additional user info
	EmailVerified bool   `json:"email_verified"`
	Username      string `json:"username"`
	FirstName     string `json:"first_name"`
	LastName      string `json:"last_name"`
}

// ExtractUserInfo extracts user information from a JWT token without verification
// This is safe because the token has already been verified by the backend
func ExtractUserInfo(tokenString string) (*JWTClaims, error) {
	// Parse without verification (backend already verified)
	parser := jwt.NewParser(jwt.WithoutClaimsValidation())
	
	token, _, err := parser.ParseUnverified(tokenString, &jwt.MapClaims{})
	if err != nil {
		return nil, fmt.Errorf("failed to parse token: %w", err)
	}
	
	claims, ok := token.Claims.(*jwt.MapClaims)
	if !ok {
		return nil, fmt.Errorf("invalid token claims")
	}
	
	// Extract claims into our struct
	jwtClaims := &JWTClaims{}
	
	// Extract standard fields
	if sub, ok := (*claims)["sub"].(string); ok {
		jwtClaims.Subject = sub
		jwtClaims.UserID = sub // WorkOS uses sub as user_id
	}
	
	if email, ok := (*claims)["email"].(string); ok {
		jwtClaims.Email = email
	}
	
	if name, ok := (*claims)["name"].(string); ok {
		jwtClaims.Name = name
	}
	
	if username, ok := (*claims)["username"].(string); ok {
		jwtClaims.Username = username
	}
	
	if firstName, ok := (*claims)["first_name"].(string); ok {
		jwtClaims.FirstName = firstName
	}
	
	if lastName, ok := (*claims)["last_name"].(string); ok {
		jwtClaims.LastName = lastName
	}
	
	if emailVerified, ok := (*claims)["email_verified"].(bool); ok {
		jwtClaims.EmailVerified = emailVerified
	}
	
	// Extract organization info
	if orgID, ok := (*claims)["org_id"].(string); ok {
		jwtClaims.OrganizationID = orgID
	}
	
	if orgIDs, ok := (*claims)["org_ids"].([]interface{}); ok {
		jwtClaims.Organizations = make([]string, 0, len(orgIDs))
		for _, id := range orgIDs {
			if strID, ok := id.(string); ok {
				jwtClaims.Organizations = append(jwtClaims.Organizations, strID)
			}
		}
	}
	
	// Extract actor type (user or machine)
	if actorType, ok := (*claims)["actor_type"].(string); ok {
		jwtClaims.ActorType = actorType
	}
	
	// Extract timestamps
	if exp, ok := (*claims)["exp"].(float64); ok {
		jwtClaims.ExpiresAt = int64(exp)
	}
	
	if iat, ok := (*claims)["iat"].(float64); ok {
		jwtClaims.IssuedAt = int64(iat)
	}
	
	return jwtClaims, nil
}

// GetDisplayName returns the best available display name for the user
func (c *JWTClaims) GetDisplayName() string {
	// Prefer username
	if c.Username != "" {
		return c.Username
	}
	
	// Then full name
	if c.Name != "" {
		return c.Name
	}
	
	// Then construct from first/last
	if c.FirstName != "" || c.LastName != "" {
		return strings.TrimSpace(c.FirstName + " " + c.LastName)
	}
	
	// Then email prefix
	if c.Email != "" {
		if at := strings.Index(c.Email, "@"); at > 0 {
			return c.Email[:at]
		}
		return c.Email
	}
	
	// Finally user ID
	if c.UserID != "" {
		return c.UserID
	}
	
	return c.Subject
}

// IsExpired checks if the token is expired
func (c *JWTClaims) IsExpired() bool {
	if c.ExpiresAt == 0 {
		return false
	}
	return time.Now().Unix() > c.ExpiresAt
}

// ExtractIDToken extracts user info from an ID token if present
func ExtractIDToken(tokenResp *TokenResponse) (*JWTClaims, error) {
	if tokenResp.IDToken == "" {
		// No ID token, try to extract from access token
		return ExtractUserInfo(tokenResp.AccessToken)
	}
	
	return ExtractUserInfo(tokenResp.IDToken)
}