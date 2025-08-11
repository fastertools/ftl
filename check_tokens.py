#!/usr/bin/env python3
import json
import base64
import subprocess
import sys

def decode_jwt(token):
    """Decode a JWT token and return its claims"""
    parts = token.split('.')
    if len(parts) >= 2:
        payload = parts[1]
        # Add padding if needed
        payload += '=' * (4 - len(payload) % 4)
        decoded = base64.urlsafe_b64decode(payload)
        return json.loads(decoded)
    return None

# Get credentials from keyring using the keyring command
try:
    import keyring
    entry = keyring.get_password('ftl-cli', 'default')
    
    if not entry:
        print("No credentials found in keyring")
        sys.exit(1)
    
    creds = json.loads(entry)
    
    print("=== CHECKING BOTH TOKENS ===\n")
    
    # Check access token
    if 'access_token' in creds:
        print("1. ACCESS TOKEN Claims:")
        claims = decode_jwt(creds['access_token'])
        print(json.dumps(claims, indent=2))
        print(f"\n   Has org_id? {'✅' if 'org_id' in claims else '❌'}")
        print(f"   Has custom claims? {'✅' if any(k not in ['iss', 'aud', 'sub', 'exp', 'iat', 'jti', 'sid'] for k in claims.keys()) else '❌'}")
    
    print("\n" + "="*50 + "\n")
    
    # Check ID token
    if 'id_token' in creds:
        print("2. ID TOKEN Claims:")
        claims = decode_jwt(creds['id_token'])
        print(json.dumps(claims, indent=2))
        print(f"\n   Has org_id? {'✅' if 'org_id' in claims else '❌'}")
        print(f"   Has custom claims? {'✅' if any(k not in ['iss', 'aud', 'sub', 'exp', 'iat', 'jti', 'sid', 'nonce'] for k in claims.keys()) else '❌'}")
    else:
        print("2. ID TOKEN: Not found in credentials")
    
except Exception as e:
    print(f"Error: {e}")
    print("\nTrying alternative method...")
    
    # Alternative: use ftl command
    try:
        result = subprocess.run(["ftl", "eng", "auth", "token"], capture_output=True, text=True)
        if result.returncode == 0:
            token = result.stdout.strip()
            print("\nAccess Token from 'ftl eng auth token':")
            claims = decode_jwt(token)
            print(json.dumps(claims, indent=2))
            print(f"\nHas org_id? {'✅' if 'org_id' in claims else '❌'}")
    except Exception as e2:
        print(f"Alternative method also failed: {e2}")