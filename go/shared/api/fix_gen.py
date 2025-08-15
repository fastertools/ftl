#!/usr/bin/env python3
import re

with open('client.gen.go', 'r') as f:
    content = f.read()

# Find and rename the model types (first occurrences) to add Body suffix
replacements = [
    (r'^(type CreateAppResponse struct)', r'type CreateAppResponseBody struct', 137),
    (r'^(type CreateDeploymentResponse struct)', r'type CreateDeploymentResponseBody struct', 211),
    (r'^(type CreateEcrTokenResponse struct)', r'type CreateEcrTokenResponseBody struct', 241),
    (r'^(type DeleteAppResponse struct)', r'type DeleteAppResponseBody struct', 265),
    (r'^(type GetAppLogsResponse struct)', r'type GetAppLogsResponseBody struct', 277),
    (r'^(type GetUserOrgsResponse struct)', r'type GetUserOrgsResponseBody struct', 293),
    (r'^(type ListAppsResponse struct)', r'type ListAppsResponseBody struct', 305),
    (r'^(type ListComponentsResponse struct)', r'type ListComponentsResponseBody struct', 332),
    (r'^(type UpdateComponentsResponse struct)', r'type UpdateComponentsResponseBody struct', 352),
]

lines = content.split('\n')

# Apply renames at specific line numbers
for pattern, replacement, line_num in replacements:
    if line_num - 1 < len(lines):
        lines[line_num - 1] = re.sub(pattern, replacement, lines[line_num - 1])

# Now update references in the response wrapper types
updated = '\n'.join(lines)

# Update JSON field references in response wrappers
updated = re.sub(r'JSON200\s+\*ListAppsResponse\b(?!Body)', r'JSON200      *ListAppsResponseBody', updated)
updated = re.sub(r'JSON201\s+\*CreateAppResponse\b(?!Body)', r'JSON201      *CreateAppResponseBody', updated)
updated = re.sub(r'JSON202\s+\*DeleteAppResponse\b(?!Body)', r'JSON202      *DeleteAppResponseBody', updated)
updated = re.sub(r'JSON200\s+\*ListComponentsResponse\b(?!Body)', r'JSON200      *ListComponentsResponseBody', updated)
updated = re.sub(r'JSON200\s+\*UpdateComponentsResponse\b(?!Body)', r'JSON200      *UpdateComponentsResponseBody', updated)
updated = re.sub(r'JSON202\s+\*CreateDeploymentResponse\b(?!Body)', r'JSON202      *CreateDeploymentResponseBody', updated)
updated = re.sub(r'JSON200\s+\*GetAppLogsResponse\b(?!Body)', r'JSON200      *GetAppLogsResponseBody', updated)
updated = re.sub(r'JSON200\s+\*CreateEcrTokenResponse\b(?!Body)', r'JSON200      *CreateEcrTokenResponseBody', updated)
updated = re.sub(r'JSON200\s+\*GetUserOrgsResponse\b(?!Body)', r'JSON200      *GetUserOrgsResponseBody', updated)

# Update var declarations in parsing functions
updated = re.sub(r'var dest ListAppsResponse\b(?!Body)', r'var dest ListAppsResponseBody', updated)
updated = re.sub(r'var dest CreateAppResponse\b(?!Body)', r'var dest CreateAppResponseBody', updated)
updated = re.sub(r'var dest DeleteAppResponse\b(?!Body)', r'var dest DeleteAppResponseBody', updated)
updated = re.sub(r'var dest ListComponentsResponse\b(?!Body)', r'var dest ListComponentsResponseBody', updated)
updated = re.sub(r'var dest UpdateComponentsResponse\b(?!Body)', r'var dest UpdateComponentsResponseBody', updated)
updated = re.sub(r'var dest CreateDeploymentResponse\b(?!Body)', r'var dest CreateDeploymentResponseBody', updated)
updated = re.sub(r'var dest GetAppLogsResponse\b(?!Body)', r'var dest GetAppLogsResponseBody', updated)
updated = re.sub(r'var dest CreateEcrTokenResponse\b(?!Body)', r'var dest CreateEcrTokenResponseBody', updated)
updated = re.sub(r'var dest GetUserOrgsResponse\b(?!Body)', r'var dest GetUserOrgsResponseBody', updated)

with open('client.gen.go', 'w') as f:
    f.write(updated)

print("Fixed duplicate type definitions")
