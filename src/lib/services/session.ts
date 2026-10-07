import { SESSION_DURATION } from '$lib/constants';
import { prisma } from '$lib/prisma';

export const createSession = async (userId: string) => {
	return await prisma.session.create({
		data: {
			userId: userId,
			expiresAt: new Date(Date.now() + SESSION_DURATION)
		}
	});
};

export const getSession = async (sessionId: string) => {
	return await prisma.session.findFirst({
		where: {
			id: sessionId
		},
		select: {
			id: true,
			expiresAt: true,
			user: {
				select: {
					id: true,
					domain: true
				}
			}
		}
	});
};

export const deleteSession = async (sessionId: string) => {
	return await prisma.session.delete({
		where: { id: sessionId }
	});
};
