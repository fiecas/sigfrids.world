import crypto from 'crypto';

export const generateVerifier = () => {
	return Buffer.from(crypto.randomBytes(32)).toString('base64url');
};

export const generateChallenge = (verifier: string) => {
	return crypto.createHash('sha256').update(verifier).digest('base64url');
};
