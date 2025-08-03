#!/usr/bin/env python3
"""
JWT Test Helper for MCP Authorizer
Generates test JWT tokens for integration testing
"""

import jwt
import json
import time
from datetime import datetime, timedelta, timezone
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.hazmat.backends import default_backend
import base64
import argparse

class JWTTestHelper:
    def __init__(self):
        # Generate RSA key pair
        self.private_key = rsa.generate_private_key(
            public_exponent=65537,
            key_size=2048,
            backend=default_backend()
        )
        self.public_key = self.private_key.public_key()
        
    def get_public_key_pem(self):
        """Get public key in PEM format"""
        return self.public_key.public_key_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PublicFormat.SubjectPublicKeyInfo
        ).decode('utf-8')
    
    def get_jwks(self, kid="test-key-1"):
        """Get JWKS representation of the public key"""
        # Get the public key numbers
        public_numbers = self.public_key.public_numbers()
        
        # Convert to base64url format
        def int_to_base64url(n):
            # Convert integer to bytes
            byte_length = (n.bit_length() + 7) // 8
            bytes_data = n.to_bytes(byte_length, byteorder='big')
            # Encode to base64url
            return base64.urlsafe_b64encode(bytes_data).decode('ascii').rstrip('=')
        
        n = int_to_base64url(public_numbers.n)
        e = int_to_base64url(public_numbers.e)
        
        return {
            "keys": [{
                "kty": "RSA",
                "use": "sig",
                "alg": "RS256",
                "kid": kid,
                "n": n,
                "e": e
            }]
        }
    
    def create_token(self, 
                    subject="test-user",
                    issuer="https://test.authkit.app",
                    audience="test-audience",
                    expires_in=3600,
                    scopes=None,
                    kid="test-key-1",
                    additional_claims=None):
        """Create a JWT token"""
        now = datetime.now(timezone.utc)
        
        claims = {
            "sub": subject,
            "iss": issuer,
            "aud": audience,
            "exp": now + timedelta(seconds=expires_in),
            "iat": now,
            "jti": f"test-{int(time.time())}"
        }
        
        if scopes:
            claims["scope"] = " ".join(scopes) if isinstance(scopes, list) else scopes
            
        if additional_claims:
            claims.update(additional_claims)
        
        # Sign the token
        token = jwt.encode(
            claims,
            self.private_key,
            algorithm="RS256",
            headers={"kid": kid}
        )
        
        return token

def main():
    parser = argparse.ArgumentParser(description='Generate JWT tokens for testing')
    parser.add_argument('--action', choices=['token', 'jwks', 'public-key'], 
                       default='token', help='Action to perform')
    parser.add_argument('--subject', default='test-user', help='Token subject')
    parser.add_argument('--issuer', default='https://test.authkit.app', help='Token issuer')
    parser.add_argument('--audience', default='test-audience', help='Token audience')
    parser.add_argument('--expires-in', type=int, default=3600, help='Token expiration in seconds')
    parser.add_argument('--scopes', nargs='+', help='Token scopes')
    parser.add_argument('--kid', default='test-key-1', help='Key ID')
    parser.add_argument('--expired', action='store_true', help='Create an expired token')
    
    args = parser.parse_args()
    
    helper = JWTTestHelper()
    
    if args.action == 'jwks':
        print(json.dumps(helper.get_jwks(args.kid), indent=2))
    elif args.action == 'public-key':
        print(helper.get_public_key_pem())
    else:  # token
        expires_in = -3600 if args.expired else args.expires_in
        token = helper.create_token(
            subject=args.subject,
            issuer=args.issuer,
            audience=args.audience,
            expires_in=expires_in,
            scopes=args.scopes,
            kid=args.kid
        )
        print(token)

if __name__ == "__main__":
    main()