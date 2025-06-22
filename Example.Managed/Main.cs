using System;

namespace Example.Managed
{

	public class ExampleClass
	{

		internal static unsafe delegate*<float, float> TestInternalCall;

		public static float StaticMethod(float value)
		{
			Console.WriteLine($"Value in C#: {value}");

			float res;
			unsafe
			{
				res = TestInternalCall(value - 10.0f);
			}
			return res;
		}

	}

}
