### Fundamental Principles of Secure Network Protocols

* ***Confidentiality*:** Ensuring that information is accessible only to those authorized to access it. It protects sensitive informaiton from unauthorized access and disclosure.
  
  * Encryption is the primary tool for maintaining confidentiality. Access controls also contribute to ensuring that data is not disclosed to unauthorized individuals or systems. 

* ***Integrity***: Maintaining and assuring the accuracy and completeness of data over its lifecycle. This means that information is not altered in unauthorized ways and that any changes are detectable. 
  
  * Integrity is achieved through controls such as checksums, cryptographic hash functions, digital signatures, and access controls that prevent unauthorized data modification. 

* ***Authentication***: The process of verifying the identity of a user, system, or entity. It ensures that the entity is who it claims to be.
  
  * Authentication is commonly achieved through methods like passwords, biometrics, and MFA. 

* ***Authorization***: Once identity is authenticated, authorization is the process of granting or denying rights and permissions to access and use resources.
  
  * Authorization is managed through access control mechanisms like role-based access control (RBAC) and permissions settings. 

* ***Non-Repudiation***: Ensures that a party in a communication cannot deny the authenticity of their signature on a document or a message that they sent. It is crucial in scenarios where proof of origin is required. 
  
  * Non-repudiation is typically achieved through digital signatures by binding a document to the signers unique identity, certificate authorities in their issuance of digital certificates which verify the ownership of public keys used in digital signatures which helps ensure the public key belongs to the person or entity claiming it, timestamping, audit trails, and logs. 

* ***Availability***: Ensures that information systems and data are accessible and usable upon demand by authorized users. 
  
  * Availability is typically achieved through redundancy, failover mechanisms, load balancing, data backup and recovery, monitoring and alers. 



### Notes on AES-GCM

The operations of GCM depend on the choice of underlying symmetric key block cipher and thus can be considered a mode of operation of the block cipher. This means that the GCM key is the block cipher key. 

The block size of the underlying block cipher shall be 128 bits and the key size shall be at least 128 bits. 

##### Input Data

Given the selection of an approved block cipher and key, there are three input strings to the authenticated encryption function:



* a plaintext, denoted P;

* additional authenticated data (AAD); and

* an initialization vector (IV), denoted IV.



The plaintext and the AAD are the two categories of data that GCM protects. GCM protects the authenticity of the plaintext and the AAD; GCM also protects the confidentiality of the plaintext, while the AAD is left in the clear. 



The IV is essentially a nonce, i.e, a value that is unique within the specified context, which determines an invocation of the authenticated encryption function on the input data to be protected. 



The bit lengths of the input strings to the authenticated encryption function shall meet the following requirements: 



* len(P) <= 2.pow(39) - 256;

* len(A) <= 2.pow(64) - 1;

* 1 <= len(IV) <= 2.pow(64) - 1;



The AAD and IV shall be multiples of 8.



##### Output Data

The following two bit strings comprise the output data of the authenticated encryption function:



* A ciphertext, denoted C, whose bit length is the same as that of the plaintext.

* An authentication tag, or tag, for short, denoted T.



The bit length of T may be any of the following five values: 128, 120, 112, 104, or 96.


