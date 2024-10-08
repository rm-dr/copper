/**
 * Navigation-blocking hack sourced from https://github.com/vercel/next.js/discussions/41934#discussioncomment-8996669.
 * How to use:
 * - Wrap everything in a `<NavigationBlockerProvider>`
 * - Use `<BlockableLink />` instead of `<Link/>` for links you want to block
 * - Render a `<NavBlocker />` when you want to prevent navigation.
 * This handles both browser and `<BlockableLink />` navigation.
 */

"use client";

import {
	Dispatch,
	SetStateAction,
	createContext,
	useContext,
	useEffect,
	useState,
	startTransition,
} from "react";
import NextLink from "next/link";
import { useRouter } from "next/navigation";

const NavigationBlockerContext = createContext<
	[isBlocked: boolean, setBlocked: Dispatch<SetStateAction<boolean>>]
>([false, () => {}]);

export function NavigationBlockerProvider({
	children,
}: {
	children: React.ReactNode;
}) {
	const state = useState(false);
	return (
		<NavigationBlockerContext.Provider value={state}>
			{children}
		</NavigationBlockerContext.Provider>
	);
}

export function useIsBlocked() {
	const [isBlocked] = useContext(NavigationBlockerContext);
	return isBlocked;
}

export function NavBlocker() {
	const [isBlocked, setBlocked] = useContext(NavigationBlockerContext);

	// Block Next navigation
	useEffect(() => {
		setBlocked(() => {
			return true;
		});
		return () => {
			setBlocked(() => {
				return false;
			});
		};
	}, [isBlocked, setBlocked]);

	// Block browser navigation
	useEffect(() => {
		if (isBlocked) {
			const showModal = (event: BeforeUnloadEvent) => {
				event.preventDefault();
			};

			window.addEventListener("beforeunload", showModal);
			return () => {
				window.removeEventListener("beforeunload", showModal);
			};
		}
	}, [isBlocked]);

	return null;
}

export function nav_confirm(): boolean {
	return window.confirm(
		"This page is asking you to confirm that you want to leave — information you’ve entered may not be saved.",
	);
}

/**
 * A drop-in replacement for Next's `<Link/>` that asks for confirmation
 * if clicked while a `<NavBlocker/>` is active.
 */
export function BlockableLink({
	href,
	children,
	replace,
	...rest
}: Parameters<typeof NextLink>[0]) {
	const router = useRouter();
	const isBlocked = useIsBlocked();

	return (
		<NextLink
			href={href}
			onClick={(e) => {
				e.preventDefault();

				// Cancel navigation
				if (isBlocked && !nav_confirm()) {
					return;
				}

				startTransition(() => {
					const url = href.toString();
					if (replace) {
						router.replace(url);
					} else {
						router.push(url);
					}
				});
			}}
			{...rest}
		>
			{children}
		</NextLink>
	);
}
