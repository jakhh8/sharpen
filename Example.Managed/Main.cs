using System;

namespace Example.Managed
{

	[AttributeUsage(AttributeTargets.Class)]
	public sealed class CustomAttribute : Attribute
	{
		public float Value;
	}

	[Custom(Value = -2500.0f)]
	public class ExampleClass
	{

		public struct MyVec3
		{
			public float X;
			public float Y;
			public float Z;
		}

		internal static unsafe delegate*<float, float> TestInternalCall;

		private int myPrivateValue;
		public int PublicProp
		{
			get => myPrivateValue;
			set => myPrivateValue = value * 2;
		}

		public ExampleClass(int someValue)
		{
			Console.WriteLine($"Example({someValue})");
		}

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

		public void MemberMethod(MyVec3 vec3)
		{
			MyVec3 anotherVector = new()
			{
				X = 10,
				Y = 20,
				Z = 30
			};

			// TODO: Icall?

			Console.WriteLine($"X: {vec3.X}, Y: {vec3.Y}, Z: {vec3.Z}");
		}

	}

}
