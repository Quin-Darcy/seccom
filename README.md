# SECCOM

This program is a simple demonstration of some of the few cryptographic principles used in secure communication systems. The prinicples covered in this demonstration are symmetric-key cryptography, Diffie-Hellman key exchange, [?] key derivation, and MAC tags. 

The relavant standards which were used to inform the writing of this demonstration are listed below:

1. Symmetric-Key Cryptography: NIST FIPS 197, ...
2. Diffie-Hellman Key Exchange: NIST SP 800-56A, Rev. 3, 
3. Key Derivation: NIST SP 800-56C, Rev. 2
4. MAC Tags: ...

Each of these principles were written as independent libraries to promote modularity and clear repsonsibility. The demonstration itself follows an evolution of communications between client and server. There are a total of 4 clients and 4 servers each to be run in pairs, one at a time. The first client and server exchange messages in plaintext and this represents a security baseline from which we will improve over the successive client and server pairs. The next pair introcudes encryption/decryption by utilizing the aes_crypt library. However, a major flaw in the communication between this pair was the insecure transmission of the cryptographic keying material used by both parties to encrypt/decrypt the messages. The next pair attempts to address this flaw by introducing a Diffie-Hellman key exchange between the client and server. This allows both parties to mutually contribute to a shared secret by exchanging public information as means of computing the same private key. The security strength of this addition relies on the intractability of the Discrete Logarithm problem. 


Security Considerations of This Demonstartion

There are several dimensions of security which are lacking in this demonstration. The RNGs used as well as potential for side-chain analysis ...