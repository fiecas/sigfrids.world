// @ts-nocheck
import { prisma } from '$lib/prisma';
import type { PageServerLoad } from './$types';

export const load = ({ params }: Parameters<PageServerLoad>[0]) => {
	const post = prisma.post.findUnique({
		where: {
			slug: params.slug
		}
	});

	return { post };
};
