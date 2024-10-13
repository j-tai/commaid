/**
 * Generates a random room name.
 */
export function generateRoom(): string {
    let result = '';
    for (let i = 0; i < 6; ++i) {
        result += ROOM_ALPHABET.charAt(Math.floor(Math.random() * ROOM_ALPHABET.length));
    }
    return result;
}

// Exclude vowels and '0' to avoid accidentally generating inappropriate words
const ROOM_ALPHABET = 'bcdfghjklmnpqrstvwxzBCDFGHJKLMNPQRSTVWXZ123456789';
