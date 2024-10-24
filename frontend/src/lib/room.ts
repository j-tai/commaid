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

// Exclude vowels, '0', and ambiguous letters
const ROOM_ALPHABET = 'bcdfghjkmnpqrstvwxzBCDFGHJKLMNPQRSTVWXZ123456789';
