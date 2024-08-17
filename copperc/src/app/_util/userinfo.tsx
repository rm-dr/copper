import { create } from "zustand";
import { devtools, persist } from "zustand/middleware";
import type {} from "@redux-devtools/extension";
import { components } from "./api/openapi";
import { createTheme, MantineThemeOverride } from "@mantine/core";
import { GeistSans } from "geist/font/sans";
import { GeistMono } from "geist/font/mono";
import { generateColorsMap } from "./generate-colors";

type UserInfoState = {
	user_info: components["schemas"]["UserInfo"] | null;
	theme: MantineThemeOverride;
	set_info: (new_info: components["schemas"]["UserInfo"]) => void;
	preview_color: (new_color: string) => void;
	set_color: (new_color: string) => void;
};

export const useUserInfoStore = create<UserInfoState>()(
	devtools(
		persist(
			(set) => ({
				user_info: null,
				theme: createTheme({
					fontFamily: GeistSans.style.fontFamily,
					fontFamilyMonospace: GeistMono.style.fontFamily,
					primaryColor: "gray",
				}),

				set_info: (new_info) =>
					set((state) => ({
						...state,
						user_info: new_info,
						theme: createTheme({
							fontFamily: GeistSans.style.fontFamily,
							fontFamilyMonospace: GeistMono.style.fontFamily,

							colors: {
								"user-color": generateColorsMap(new_info.color).colors.map(
									(x) => x.toString(),
								) as any,
							},

							primaryColor: "user-color",
						}),
					})),

				preview_color: (new_color) =>
					set((state) => {
						return {
							...state,
							theme: createTheme({
								fontFamily: GeistSans.style.fontFamily,
								fontFamilyMonospace: GeistMono.style.fontFamily,

								colors: {
									"user-color": generateColorsMap(new_color).colors.map((x) =>
										x.toString(),
									) as any,
								},

								primaryColor: "user-color",
							}),
						};
					}),

				set_color: (new_color) =>
					set((state) => {
						return {
							...state,
							user_info:
								state.user_info === null
									? null
									: { ...state.user_info, color: new_color },

							theme: createTheme({
								fontFamily: GeistSans.style.fontFamily,
								fontFamilyMonospace: GeistMono.style.fontFamily,

								colors: {
									"user-color": generateColorsMap(new_color).colors.map((x) =>
										x.toString(),
									) as any,
								},

								primaryColor: "user-color",
							}),
						};
					}),
			}),

			{
				name: "user-info",
			},
		),
	),
);
