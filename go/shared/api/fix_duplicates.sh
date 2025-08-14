#!/bin/bash

# Fix CreateAppResponse
sed -i '123s/type CreateAppResponse struct/type CreateAppResponseBody struct/' client.gen.go
sed -i '1359s/JSON201      \*CreateAppResponse/JSON201      *CreateAppResponseBody/' client.gen.go
sed -i '1767s/var dest CreateAppResponse/var dest CreateAppResponseBody/' client.gen.go

# Fix CreateDeploymentResponse  
sed -i '171s/type CreateDeploymentResponse struct/type CreateDeploymentResponseBody struct/' client.gen.go
sed -i '1486s/JSON202      \*CreateDeploymentResponse/JSON202      *CreateDeploymentResponseBody/' client.gen.go
sed -i '2016s/var dest CreateDeploymentResponse/var dest CreateDeploymentResponseBody/' client.gen.go

# Fix CreateEcrTokenResponse
sed -i '186s/type CreateEcrTokenResponse struct/type CreateEcrTokenResponseBody struct/' client.gen.go
sed -i '1538s/JSON200      \*CreateEcrTokenResponse/JSON200      *CreateEcrTokenResponseBody/' client.gen.go
sed -i '2124s/var dest CreateEcrTokenResponse/var dest CreateEcrTokenResponseBody/' client.gen.go

# Fix DeleteAppResponse
sed -i '195s/type DeleteAppResponse struct/type DeleteAppResponseBody struct/' client.gen.go
sed -i '1385s/JSON202      \*DeleteAppResponse/JSON202      *DeleteAppResponseBody/' client.gen.go
sed -i '1821s/var dest DeleteAppResponse/var dest DeleteAppResponseBody/' client.gen.go

# Fix GetAppLogsResponse
sed -i '207s/type GetAppLogsResponse struct/type GetAppLogsResponseBody struct/' client.gen.go
sed -i '1511s/JSON200      \*GetAppLogsResponse/JSON200      *GetAppLogsResponseBody/' client.gen.go
sed -i '2063s/var dest GetAppLogsResponse/var dest GetAppLogsResponseBody/' client.gen.go

# Fix GetUserOrgsResponse
sed -i '223s/type GetUserOrgsResponse struct/type GetUserOrgsResponseBody struct/' client.gen.go
sed -i '1562s/JSON200      \*GetUserOrgsResponse/JSON200      *GetUserOrgsResponseBody/' client.gen.go
sed -i '2164s/var dest GetUserOrgsResponse/var dest GetUserOrgsResponseBody/' client.gen.go

# Fix UpdateComponentsResponse
sed -i '282s/type UpdateComponentsResponse struct/type UpdateComponentsResponseBody struct/' client.gen.go
sed -i '1460s/JSON200      \*UpdateComponentsResponse/JSON200      *UpdateComponentsResponseBody/' client.gen.go
sed -i '1962s/var dest UpdateComponentsResponse/var dest UpdateComponentsResponseBody/' client.gen.go

