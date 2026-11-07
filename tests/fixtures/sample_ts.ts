// TypeScript sample fixture for ctx-prune verification

/**
 * Interface representing a user session token.
 */
export interface UserSession {
    userId: string;
    roles: string[];
    expiresAt: number;
}

/* Multi-line comment explaining service logic */
export class AuthService {
    private secretKey: string;

    constructor(secretKey: string) {
        // Store secret key securely
        this.secretKey = secretKey;
    }

    public generateToken(user: UserSession): string {
        /* Template literal containing comment-like strings */
        const template = `Token for user: ${user.userId} // not a comment /* keep */`;
        return `${template}.${this.secretKey}`;
    }
}
