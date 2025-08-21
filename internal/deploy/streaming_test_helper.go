package deploy

import (
	"github.com/fastertools/ftl-cli/internal/api"
)

// Helper function to create test credentials
func createTestCredentials(functionURL, registryURI, actorType, userID string, orgIDs []string) *api.CreateDeployCredentialsResponseBody {
	creds := &api.CreateDeployCredentialsResponseBody{
		Registry: struct {
			AuthorizationToken string `json:"authorizationToken"`
			ExpiresAt          string `json:"expiresAt"`
			PackageNamespace   string `json:"packageNamespace"`
			ProxyEndpoint      string `json:"proxyEndpoint"`
			Region             string `json:"region"`
			RegistryUri        string `json:"registryUri"`
		}{
			RegistryUri: registryURI,
			Region:      "us-west-2",
		},
	}

	// Set deployment fields based on whether we have context
	if actorType != "" {
		var actorTypeEnum api.CreateDeployCredentialsResponseBodyDeploymentContextActorType
		if actorType == "user" {
			actorTypeEnum = api.User
		} else {
			actorTypeEnum = api.Machine
		}

		creds.Deployment = struct {
			Context struct {
				ActorType api.CreateDeployCredentialsResponseBodyDeploymentContextActorType `json:"actorType"`
				OrgIds    []string                                                          `json:"orgIds"`
				UserId    string                                                            `json:"userId"`
			} `json:"context"`
			Credentials struct {
				AccessKeyId     string `json:"accessKeyId"`
				ExpiresAt       string `json:"expiresAt"`
				SecretAccessKey string `json:"secretAccessKey"`
				SessionToken    string `json:"sessionToken"`
			} `json:"credentials"`
			FunctionUrl string `json:"functionUrl"`
		}{
			FunctionUrl: functionURL,
			Context: struct {
				ActorType api.CreateDeployCredentialsResponseBodyDeploymentContextActorType `json:"actorType"`
				OrgIds    []string                                                          `json:"orgIds"`
				UserId    string                                                            `json:"userId"`
			}{
				ActorType: actorTypeEnum,
				UserId:    userID,
				OrgIds:    orgIDs,
			},
			Credentials: struct {
				AccessKeyId     string `json:"accessKeyId"`
				ExpiresAt       string `json:"expiresAt"`
				SecretAccessKey string `json:"secretAccessKey"`
				SessionToken    string `json:"sessionToken"`
			}{
				AccessKeyId:     "AKIAIOSFODNN7EXAMPLE",
				SecretAccessKey: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
				SessionToken:    "test-session-token",
			},
		}
	} else {
		// No context (for older tests)
		creds.Deployment = struct {
			Context struct {
				ActorType api.CreateDeployCredentialsResponseBodyDeploymentContextActorType `json:"actorType"`
				OrgIds    []string                                                          `json:"orgIds"`
				UserId    string                                                            `json:"userId"`
			} `json:"context"`
			Credentials struct {
				AccessKeyId     string `json:"accessKeyId"`
				ExpiresAt       string `json:"expiresAt"`
				SecretAccessKey string `json:"secretAccessKey"`
				SessionToken    string `json:"sessionToken"`
			} `json:"credentials"`
			FunctionUrl string `json:"functionUrl"`
		}{
			FunctionUrl: functionURL,
			Credentials: struct {
				AccessKeyId     string `json:"accessKeyId"`
				ExpiresAt       string `json:"expiresAt"`
				SecretAccessKey string `json:"secretAccessKey"`
				SessionToken    string `json:"sessionToken"`
			}{
				AccessKeyId:     "AKIAIOSFODNN7EXAMPLE",
				SecretAccessKey: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
				SessionToken:    "test-session-token",
			},
		}
	}

	return creds
}
