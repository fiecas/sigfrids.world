// @ts-nocheck
import { prisma } from '$lib/prisma';
import type { PageServerLoad } from './$types';

export const load = async () => {
	const posts = await prisma.post.findMany({ select: { title: true, slug: true } });

	return { posts };
};
;null as any as PageServerLoad;