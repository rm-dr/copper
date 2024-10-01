"use client";

import { Modal } from "@mantine/core";
import { ReactNode } from "react";
import modalStyle from "./modalbase.module.scss";

export { modalStyle };

/**
 * Base for small modals
 */
export function ModalBaseSmall(params: {
	children: ReactNode;
	title: ReactNode;
	keepOpen?: boolean;
	close: () => void;
	opened: boolean;
	hardtoclose?: boolean;
}) {
	return (
		<Modal
			closeOnClickOutside={!params.hardtoclose === true}
			closeOnEscape={!params.hardtoclose === true}
			opened={params.opened}
			onClose={() => {
				if (params.keepOpen !== true) {
					params.close();
				}
			}}
			title={params.title}
			centered
			overlayProps={{
				backgroundOpacity: 0.5,
				blur: 1,
			}}
			size={"30rem"}
		>
			{params.children}
		</Modal>
	);
}
