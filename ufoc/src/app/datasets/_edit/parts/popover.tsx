import { XIconX } from "@/app/components/icons";
import { ActionIcon, Popover } from "@mantine/core";
import { ReactNode, useEffect, useRef, useState } from "react";

export function ButtonPopover(params: {
	color: string;
	icon: ReactNode;
	children: ReactNode;
	isLoading: boolean;
	isOpened: boolean;
	setOpened: (opened: boolean) => void;
}) {
	const ref = useRef(null);

	// Close on click out (since)
	useEffect(() => {
		function onClickOutside(event: any) {
			// TODO: fix this type error
			if (ref.current && !ref.current.contains(event.target)) {
				params.setOpened(false);
			}
		}
		document.addEventListener("mousedown", onClickOutside);
		return () => {
			document.removeEventListener("mousedown", onClickOutside);
		};
	}, [ref, params]);

	return (
		<Popover
			position="bottom"
			withArrow
			shadow="md"
			trapFocus
			width="20rem"
			opened={params.isOpened}
			onChange={(b) => {
				params.setOpened(b);
			}}
		>
			<Popover.Target>
				<ActionIcon
					loading={params.isLoading}
					variant="light"
					color={params.isOpened ? "red" : params.color}
					style={{ cursor: "default" }}
					onClick={() => {
						params.setOpened(!params.isOpened);
					}}
				>
					{params.isOpened ? (
						<XIconX style={{ width: "70%", height: "70%" }} />
					) : (
						params.icon
					)}
				</ActionIcon>
			</Popover.Target>
			<Popover.Dropdown ref={ref}>{params.children}</Popover.Dropdown>
		</Popover>
	);
}
