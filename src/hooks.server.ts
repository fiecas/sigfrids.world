import { SESSION_COOKIE_NAME } from '$lib/constants';
import { getSession } from '$lib/services/session';
import type { Handle } from '@sveltejs/kit';

export const handle: Handle = async ({ event, resolve }) => {
	const sessionId = event.cookies.get(SESSION_COOKIE_NAME);
	if (!sessionId) {
		event.locals.auth = null;
		return resolve(event);
	}

	const sessionWithUser = await getSession(sessionId);
	if (!sessionWithUser || sessionWithUser.expiresAt < new Date(Date.now())) {
		event.locals.auth = null;
		return resolve(event);
	}

	const { user, ...session } = sessionWithUser;
	event.locals.auth = { session, user };

	return resolve(event);
};
